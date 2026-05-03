#[async_trait::async_trait]
pub trait JtiBlacklistService: Send + Sync {
    async fn add(&self, jti: uuid::Uuid, expires_at: chrono::DateTime<chrono::Utc>) -> Result<(), anyhow::Error>;
    async fn is_blacklisted(&self, jti: uuid::Uuid) -> Result<bool, anyhow::Error>;
}

pub struct JtiBlacklistServiceImpl {}

impl JtiBlacklistServiceImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl JtiBlacklistService for JtiBlacklistServiceImpl {
    async fn add(&self, jti: uuid::Uuid, expires_at: chrono::DateTime<chrono::Utc>) -> Result<(), anyhow::Error> {
        todo!()
    }

    async fn is_blacklisted(&self, jti: uuid::Uuid) -> Result<bool, anyhow::Error> {
        todo!()
    }
}
