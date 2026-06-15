use anyhow::Context;
use base64::Engine;
use hmac::{Hmac, Mac};
use md5::Md5;
use reqwest::Method;
use sha2::{Digest, Sha256};

pub struct PresignedPut {
    pub url: String,
    pub method: String,
    pub content_type: String,
    pub content_length: i64,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

pub struct PresignedGet {
    pub url: String,
    pub method: String,
    pub content_type: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

pub struct HeadObject {
    pub content_type: String,
    pub content_length: i64,
}

pub struct GetObject {
    pub content_type: String,
    pub content_length: i64,
    pub body: Vec<u8>,
}

#[async_trait::async_trait]
pub trait StorageService: Send + Sync {
    async fn presign_put(
        &self,
        storage_id: uuid::Uuid,
        content_type: &str,
        content_length: Option<i64>,
        expires_in: std::time::Duration,
    ) -> Result<PresignedPut, anyhow::Error>;

    async fn presign_get(
        &self,
        storage_id: uuid::Uuid,
        expires_in: std::time::Duration,
    ) -> Result<PresignedGet, anyhow::Error>;

    async fn head(&self, storage_id: uuid::Uuid) -> Result<HeadObject, anyhow::Error>;

    async fn get_object(&self, storage_id: uuid::Uuid) -> Result<GetObject, anyhow::Error>;

    async fn put_object(
        &self,
        storage_id: uuid::Uuid,
        content_type: &str,
        body: Vec<u8>,
    ) -> Result<HeadObject, anyhow::Error>;

    async fn delete(&self, storage_id: uuid::Uuid) -> Result<(), anyhow::Error>;
}

const ALGORITHM: &str = "AWS4-HMAC-SHA256";
const SERVICE: &str = "s3";
const AWS4_REQUEST: &str = "aws4_request";
const UNSIGNED_PAYLOAD: &str = "UNSIGNED-PAYLOAD";

pub struct StorageServiceImpl {
    client: reqwest::Client,
    /// scheme://host[:port] with the scheme default port stripped, no trailing slash.
    endpoint: String,
    /// host[:port] used for the signed `host` header (must match what is sent on the wire).
    host: String,
    region: String,
    access_key: String,
    secret_key: String,
    bucket: String,
}

impl StorageServiceImpl {
    pub fn new(
        endpoint_url: String,
        region: String,
        access_key_id: String,
        secret_access_key: String,
        _provider_name: String,
        bucket: String,
    ) -> Result<Self, anyhow::Error> {
        let parsed = url::Url::parse(&endpoint_url)
            .with_context(|| format!("invalid s3 endpoint url: {endpoint_url}"))?;
        let host_str = parsed
            .host_str()
            .context("s3 endpoint url has no host")?
            .to_string();
        let host = match parsed.port() {
            Some(port) => format!("{host_str}:{port}"),
            None => host_str,
        };
        let endpoint = format!("{}://{}", parsed.scheme(), host);

        Ok(Self {
            client: reqwest::Client::new(),
            endpoint,
            host,
            region,
            access_key: access_key_id,
            secret_key: secret_access_key,
            bucket,
        })
    }

    /// Send a SigV4 header-signed request to the bucket/object. `subresource` is an
    /// optional query subresource such as `cors` or `lifecycle`.
    async fn send_signed(
        &self,
        method: Method,
        path: &str,
        subresource: Option<&str>,
        extra_headers: &[(&str, String)],
        body: Vec<u8>,
    ) -> Result<reqwest::Response, anyhow::Error> {
        let now = chrono::Utc::now();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date_stamp = now.format("%Y%m%d").to_string();
        let scope = format!("{date_stamp}/{}/{SERVICE}/{AWS4_REQUEST}", self.region);
        let payload_hash = sha256_hex(&body);

        // Headers covered by the signature (lowercased, sorted by name).
        let mut headers: Vec<(String, String)> = vec![
            ("host".to_string(), self.host.clone()),
            ("x-amz-content-sha256".to_string(), payload_hash.clone()),
            ("x-amz-date".to_string(), amz_date.clone()),
        ];
        for (name, value) in extra_headers {
            headers.push((name.to_lowercase(), value.clone()));
        }
        headers.sort_by(|a, b| a.0.cmp(&b.0));
        let canonical_headers: String = headers
            .iter()
            .map(|(name, value)| format!("{name}:{}\n", value.trim()))
            .collect();
        let signed_headers = headers
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>()
            .join(";");

        let canonical_query = match subresource {
            Some(name) => format!("{name}="),
            None => String::new(),
        };

        let canonical_request = format!(
            "{}\n{path}\n{canonical_query}\n{canonical_headers}\n{signed_headers}\n{payload_hash}",
            method.as_str()
        );
        let string_to_sign = format!(
            "{ALGORITHM}\n{amz_date}\n{scope}\n{}",
            sha256_hex(canonical_request.as_bytes())
        );
        let signature = hex::encode(hmac_sha256(
            &signing_key(&self.secret_key, &date_stamp, &self.region),
            string_to_sign.as_bytes(),
        ));
        let authorization = format!(
            "{ALGORITHM} Credential={}/{scope}, SignedHeaders={signed_headers}, Signature={signature}",
            self.access_key
        );

        let url = match subresource {
            Some(name) => format!("{}{path}?{name}", self.endpoint),
            None => format!("{}{path}", self.endpoint),
        };

        // `host` is set by reqwest from the URL; the x-amz-* headers are set explicitly.
        let mut builder = self
            .client
            .request(method, &url)
            .header("x-amz-content-sha256", &payload_hash)
            .header("x-amz-date", &amz_date)
            .header("authorization", authorization);
        for (name, value) in extra_headers {
            builder = builder.header(*name, value);
        }
        if !body.is_empty() {
            builder = builder.body(body);
        }

        builder.send().await.map_err(Into::into)
    }

    /// Build a SigV4 query-signed (presigned) URL. Only the `host` header is signed and
    /// the payload is unsigned, so any client can issue the request with that single URL.
    fn presign(&self, method: &str, key: &str, expires_in: std::time::Duration) -> String {
        let now = chrono::Utc::now();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date_stamp = now.format("%Y%m%d").to_string();
        let scope = format!("{date_stamp}/{}/{SERVICE}/{AWS4_REQUEST}", self.region);
        let credential = aws_encode(&format!("{}/{scope}", self.access_key));
        let expires = expires_in.as_secs().to_string();

        // Query params must be in sorted order; these keys are already alphabetical.
        let canonical_query = format!(
            "X-Amz-Algorithm={ALGORITHM}\
             &X-Amz-Credential={credential}\
             &X-Amz-Date={amz_date}\
             &X-Amz-Expires={expires}\
             &X-Amz-SignedHeaders=host"
        );
        let path = format!("/{}/{key}", self.bucket);
        let canonical_request = format!(
            "{method}\n{path}\n{canonical_query}\nhost:{}\n\nhost\n{UNSIGNED_PAYLOAD}",
            self.host
        );
        let string_to_sign = format!(
            "{ALGORITHM}\n{amz_date}\n{scope}\n{}",
            sha256_hex(canonical_request.as_bytes())
        );
        let signature = hex::encode(hmac_sha256(
            &signing_key(&self.secret_key, &date_stamp, &self.region),
            string_to_sign.as_bytes(),
        ));

        format!(
            "{}{path}?{canonical_query}&X-Amz-Signature={signature}",
            self.endpoint
        )
    }

    pub async fn set_cors(&self, allowed_origin: &str) -> Result<(), anyhow::Error> {
        let body = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?><CORSConfiguration><CORSRule><AllowedOrigin>{}</AllowedOrigin><AllowedMethod>GET</AllowedMethod><AllowedMethod>PUT</AllowedMethod><AllowedMethod>HEAD</AllowedMethod><AllowedHeader>*</AllowedHeader><ExposeHeader>ETag</ExposeHeader><MaxAgeSeconds>3000</MaxAgeSeconds></CORSRule></CORSConfiguration>"#,
            xml_escape(allowed_origin)
        )
        .into_bytes();

        self.put_bucket_subresource("cors", body).await?;

        tracing::info!(bucket = %self.bucket, allowed_origin, "configured s3 bucket cors");

        Ok(())
    }

    pub async fn set_expiration_days(&self, days: i32) -> Result<(), anyhow::Error> {
        let body = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?><LifecycleConfiguration><Rule><ID>expire-after-{days}-days</ID><Filter></Filter><Status>Enabled</Status><Expiration><Days>{days}</Days></Expiration><AbortIncompleteMultipartUpload><DaysAfterInitiation>{days}</DaysAfterInitiation></AbortIncompleteMultipartUpload></Rule></LifecycleConfiguration>"#
        )
        .into_bytes();

        self.put_bucket_subresource("lifecycle", body).await?;

        tracing::info!(bucket = %self.bucket, days, "configured s3 bucket lifecycle expiration");

        Ok(())
    }

    async fn put_bucket_subresource(
        &self,
        subresource: &str,
        body: Vec<u8>,
    ) -> Result<(), anyhow::Error> {
        let content_md5 = base64::engine::general_purpose::STANDARD.encode(Md5::digest(&body));
        let path = format!("/{}", self.bucket);
        let resp = self
            .send_signed(
                Method::PUT,
                &path,
                Some(subresource),
                &[
                    ("content-type", "application/xml".to_string()),
                    ("content-md5", content_md5),
                ],
                body,
            )
            .await?;
        ensure_success(resp).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl StorageService for StorageServiceImpl {
    async fn presign_put(
        &self,
        storage_id: uuid::Uuid,
        content_type: &str,
        content_length: Option<i64>,
        expires_in: std::time::Duration,
    ) -> Result<PresignedPut, anyhow::Error> {
        let expires_at = chrono::Utc::now() + expires_in;
        let url = self.presign("PUT", &storage_id.to_string(), expires_in);

        Ok(PresignedPut {
            url,
            method: "PUT".to_string(),
            content_type: content_type.to_string(),
            content_length: content_length.unwrap_or(-1),
            expires_at,
        })
    }

    async fn presign_get(
        &self,
        storage_id: uuid::Uuid,
        expires_in: std::time::Duration,
    ) -> Result<PresignedGet, anyhow::Error> {
        let expires_at = chrono::Utc::now() + expires_in;
        let head = self.head(storage_id).await?;
        let url = self.presign("GET", &storage_id.to_string(), expires_in);

        Ok(PresignedGet {
            url,
            method: "GET".to_string(),
            content_type: head.content_type,
            expires_at,
        })
    }

    async fn head(&self, storage_id: uuid::Uuid) -> Result<HeadObject, anyhow::Error> {
        let path = format!("/{}/{storage_id}", self.bucket);
        let resp = self
            .send_signed(Method::HEAD, &path, None, &[], Vec::new())
            .await?;
        let resp = ensure_success(resp).await?;

        let content_type =
            header(&resp, "content-type").ok_or_else(|| anyhow::anyhow!("missing content-type"))?;
        let content_length = header(&resp, "content-length")
            .and_then(|v| v.parse::<i64>().ok())
            .ok_or_else(|| anyhow::anyhow!("missing content-length"))?;

        Ok(HeadObject {
            content_type,
            content_length,
        })
    }

    async fn get_object(&self, storage_id: uuid::Uuid) -> Result<GetObject, anyhow::Error> {
        let path = format!("/{}/{storage_id}", self.bucket);
        let resp = self
            .send_signed(Method::GET, &path, None, &[], Vec::new())
            .await?;
        let resp = ensure_success(resp).await?;

        let content_type =
            header(&resp, "content-type").ok_or_else(|| anyhow::anyhow!("missing content-type"))?;
        let content_length = header(&resp, "content-length").and_then(|v| v.parse::<i64>().ok());
        let body = resp.bytes().await?.to_vec();
        let content_length = content_length.unwrap_or(body.len() as i64);

        Ok(GetObject {
            content_type,
            content_length,
            body,
        })
    }

    async fn put_object(
        &self,
        storage_id: uuid::Uuid,
        content_type: &str,
        body: Vec<u8>,
    ) -> Result<HeadObject, anyhow::Error> {
        let content_length = body.len() as i64;
        let path = format!("/{}/{storage_id}", self.bucket);

        let result = async {
            let resp = self
                .send_signed(
                    Method::PUT,
                    &path,
                    None,
                    &[("content-type", content_type.to_string())],
                    body,
                )
                .await?;
            ensure_success(resp).await?;
            Ok::<(), anyhow::Error>(())
        }
        .await;

        if let Err(e) = &result {
            tracing::error!(
                error = %e,
                content_length,
                content_type,
                bucket = %self.bucket,
                "s3 put_object failed"
            );
        }
        result?;

        Ok(HeadObject {
            content_type: content_type.to_string(),
            content_length,
        })
    }

    async fn delete(&self, storage_id: uuid::Uuid) -> Result<(), anyhow::Error> {
        let path = format!("/{}/{storage_id}", self.bucket);
        let resp = self
            .send_signed(Method::DELETE, &path, None, &[], Vec::new())
            .await?;
        ensure_success(resp).await?;
        Ok(())
    }
}

fn header(resp: &reqwest::Response, name: &str) -> Option<String> {
    resp.headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string())
}

async fn ensure_success(resp: reqwest::Response) -> Result<reqwest::Response, anyhow::Error> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }
    let body = resp.text().await.unwrap_or_default();
    anyhow::bail!("s3 request failed: {status} {body}");
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = <Hmac<Sha256>>::new_from_slice(key).expect("hmac accepts keys of any length");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn sha256_hex(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

fn signing_key(secret: &str, date_stamp: &str, region: &str) -> Vec<u8> {
    let k_date = hmac_sha256(format!("AWS4{secret}").as_bytes(), date_stamp.as_bytes());
    let k_region = hmac_sha256(&k_date, region.as_bytes());
    let k_service = hmac_sha256(&k_region, SERVICE.as_bytes());
    hmac_sha256(&k_service, AWS4_REQUEST.as_bytes())
}

/// RFC 3986 percent-encoding as required by SigV4 (unreserved set is left untouched).
fn aws_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn xml_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
