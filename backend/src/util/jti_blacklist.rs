#[async_trait::async_trait]
pub trait JtiBlacklistService: Send + Sync {
    async fn add(
        &self,
        jti: uuid::Uuid,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), anyhow::Error>;
    async fn is_blacklisted(&self, jti: uuid::Uuid) -> Result<bool, anyhow::Error>;
}

pub struct JtiBlacklistServiceImpl {
    pool: deadpool_redis::Pool,
}

impl JtiBlacklistServiceImpl {
    pub fn new(pool: deadpool_redis::Pool) -> Self {
        Self { pool }
    }

    fn key(jti: uuid::Uuid) -> String {
        format!("jti_blacklist:{jti}")
    }
}

#[async_trait::async_trait]
impl JtiBlacklistService for JtiBlacklistServiceImpl {
    async fn add(
        &self,
        jti: uuid::Uuid,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), anyhow::Error> {
        use redis::AsyncCommands as _;

        let ttl_secs = (expires_at - chrono::Utc::now()).num_seconds();
        if ttl_secs <= 0 {
            return Ok(());
        }

        let mut conn = self.pool.get().await?;
        let _: () = conn.set_ex(Self::key(jti), 1u8, ttl_secs as u64).await?;
        Ok(())
    }

    async fn is_blacklisted(&self, jti: uuid::Uuid) -> Result<bool, anyhow::Error> {
        use redis::AsyncCommands as _;

        let mut conn = self.pool.get().await?;
        let exists: bool = conn.exists(Self::key(jti)).await?;
        Ok(exists)
    }
}
