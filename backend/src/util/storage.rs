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
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

pub struct HeadObject {
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
        content_length: i64,
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

pub struct StorageServiceImpl {
    client: aws_sdk_s3::Client,
    bucket: String,
}

impl StorageServiceImpl {
    pub fn new(
        endpoint_url: String,
        region: String,
        access_key_id: String,
        secret_access_key: String,
        provider_name: String,
        bucket: String,
    ) -> Result<Self, anyhow::Error> {
        let credentials = aws_sdk_s3::config::Credentials::new(
            access_key_id,
            secret_access_key,
            None,
            None,
            Box::leak(provider_name.into_boxed_str()),
        );

        let config = aws_sdk_s3::Config::builder()
            .endpoint_url(endpoint_url)
            .credentials_provider(credentials)
            .region(aws_sdk_s3::config::Region::new(region))
            .force_path_style(true)
            .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
            .build();

        let client = aws_sdk_s3::Client::from_conf(config);

        Ok(Self { client, bucket })
    }

    pub async fn set_expiration_days(&self, days: i32) -> Result<(), anyhow::Error> {
        let rule = aws_sdk_s3::types::LifecycleRule::builder()
            .id(format!("expire-after-{days}-days"))
            .status(aws_sdk_s3::types::ExpirationStatus::Enabled)
            .filter(aws_sdk_s3::types::LifecycleRuleFilter::builder().build())
            .expiration(
                aws_sdk_s3::types::LifecycleExpiration::builder()
                    .days(days)
                    .build(),
            )
            .abort_incomplete_multipart_upload(
                aws_sdk_s3::types::AbortIncompleteMultipartUpload::builder()
                    .days_after_initiation(days)
                    .build(),
            )
            .build()?;

        let configuration = aws_sdk_s3::types::BucketLifecycleConfiguration::builder()
            .rules(rule)
            .build()?;

        self.client
            .put_bucket_lifecycle_configuration()
            .bucket(&self.bucket)
            .lifecycle_configuration(configuration)
            .send()
            .await?;

        tracing::info!(bucket = %self.bucket, days, "configured s3 bucket lifecycle expiration");

        Ok(())
    }
}

#[async_trait::async_trait]
impl StorageService for StorageServiceImpl {
    async fn presign_put(
        &self,
        storage_id: uuid::Uuid,
        content_type: &str,
        content_length: i64,
        expires_in: std::time::Duration,
    ) -> Result<PresignedPut, anyhow::Error> {
        let expires_at = chrono::Utc::now() + expires_in;
        let presigning_config = aws_sdk_s3::presigning::PresigningConfig::expires_in(expires_in)?;

        let presigned = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(storage_id.to_string())
            .content_type(content_type)
            .content_length(content_length)
            .presigned(presigning_config)
            .await?;

        Ok(PresignedPut {
            url: presigned.uri().to_string(),
            method: presigned.method().to_string(),
            content_type: content_type.to_string(),
            content_length,
            expires_at,
        })
    }

    async fn presign_get(
        &self,
        storage_id: uuid::Uuid,
        expires_in: std::time::Duration,
    ) -> Result<PresignedGet, anyhow::Error> {
        let expires_at = chrono::Utc::now() + expires_in;
        let presigning_config = aws_sdk_s3::presigning::PresigningConfig::expires_in(expires_in)?;

        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(storage_id.to_string())
            .presigned(presigning_config)
            .await?;

        Ok(PresignedGet {
            url: presigned.uri().to_string(),
            method: presigned.method().to_string(),
            expires_at,
        })
    }

    async fn head(&self, storage_id: uuid::Uuid) -> Result<HeadObject, anyhow::Error> {
        let output = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(storage_id.to_string())
            .send()
            .await?;

        let content_length = output
            .content_length()
            .ok_or_else(|| anyhow::anyhow!("missing content-length"))?;

        Ok(HeadObject { content_length })
    }

    async fn get_object(&self, storage_id: uuid::Uuid) -> Result<GetObject, anyhow::Error> {
        let output = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(storage_id.to_string())
            .send()
            .await?;

        let content_type = output
            .content_type()
            .ok_or_else(|| anyhow::anyhow!("missing content-type"))?
            .to_string();
        let content_length = output
            .content_length()
            .ok_or_else(|| anyhow::anyhow!("missing content-length"))?;
        let body = output.body.collect().await?.to_vec();

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

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(storage_id.to_string())
            .content_type(content_type)
            .content_length(content_length)
            .body(body.into())
            .send()
            .await?;

        Ok(HeadObject { content_length })
    }

    async fn delete(&self, storage_id: uuid::Uuid) -> Result<(), anyhow::Error> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(storage_id.to_string())
            .send()
            .await?;

        Ok(())
    }
}
