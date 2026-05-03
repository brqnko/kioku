#[derive(serde::Serialize, serde::Deserialize)]
struct Claims {
    iss: String,
    sub: String,
    jti: String,
    iat: u64,
    nbf: u64,
    exp: u64,
}

pub struct VerifiedUserAccessToken {
    pub user_id: uuid::Uuid,
    pub jti: uuid::Uuid,
}

pub trait JWTService: Send + Sync {
    fn sign_user_access_token(&self, user_id: uuid::Uuid, jti: uuid::Uuid, duration: chrono::Duration) -> Result<(String, chrono::DateTime<chrono::Utc>), anyhow::Error>;
    fn verify_user_access_token(&self, token: String) -> Result<VerifiedUserAccessToken, anyhow::Error>;
}

pub struct JWTServiceImpl {
    issuer: String,
    encoding_key: jsonwebtoken::EncodingKey,
    decoding_key: jsonwebtoken::DecodingKey,
}

impl JWTServiceImpl {
    pub fn new(issuer: String, private_key_pem: &[u8], public_key_pem: &[u8]) -> Result<Self, anyhow::Error> {
        let encoding_key = jsonwebtoken::EncodingKey::from_ed_pem(private_key_pem)?;
        let decoding_key = jsonwebtoken::DecodingKey::from_ed_pem(public_key_pem)?;
        Ok(Self { issuer, encoding_key, decoding_key })
    }
}

impl JWTService for JWTServiceImpl {
    fn sign_user_access_token(&self, user_id: uuid::Uuid, jti: uuid::Uuid, duration: chrono::Duration) -> Result<(String, chrono::DateTime<chrono::Utc>), anyhow::Error> {
        let now = chrono::Utc::now();
        let expires_at = now + duration;
        let exp = expires_at.timestamp() as u64;
        let iat = now.timestamp() as u64;

        let claims = Claims {
            iss: self.issuer.clone(),
            sub: user_id.to_string(),
            jti: jti.to_string(),
            iat,
            nbf: iat,
            exp,
        };

        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::EdDSA),
            &claims,
            &self.encoding_key,
        )?;

        Ok((token, expires_at))
    }

    fn verify_user_access_token(&self, token: String) -> Result<VerifiedUserAccessToken, anyhow::Error> {
        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::EdDSA);
        validation.set_issuer(&[&self.issuer]);

        let data = jsonwebtoken::decode::<Claims>(&token, &self.decoding_key, &validation)?;

        Ok(VerifiedUserAccessToken {
            user_id: data.claims.sub.parse()?,
            jti: data.claims.jti.parse()?,
        })
    }
}

#[cfg(test)]
mod tests {

}
