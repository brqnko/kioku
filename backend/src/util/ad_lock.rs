#[async_trait::async_trait]
pub trait Locker: Send + Sync {
    async fn acquire(
        &self,
        conn: &mut sqlx::MySqlConnection,
        name: &str,
        timeout_secs: u32,
    ) -> Result<(), anyhow::Error>;
    async fn release(
        &self,
        conn: &mut sqlx::MySqlConnection,
        name: &str,
    ) -> Result<(), anyhow::Error>;
}

pub struct MySqlLocker;

impl MySqlLocker {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MySqlLocker {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Locker for MySqlLocker {
    async fn acquire(
        &self,
        conn: &mut sqlx::MySqlConnection,
        name: &str,
        timeout_secs: u32,
    ) -> Result<(), anyhow::Error> {
        let result: Option<i64> = sqlx::query_scalar("SELECT GET_LOCK(?, ?)")
            .bind(sha256::digest(name))
            .bind(timeout_secs as i64)
            .fetch_one(&mut *conn)
            .await?;
        match result {
            Some(1) => Ok(()),
            Some(0) => Err(anyhow::anyhow!("ad_lock timeout: {name}")),
            _ => Err(anyhow::anyhow!("ad_lock failed: {name}")),
        }
    }

    async fn release(
        &self,
        conn: &mut sqlx::MySqlConnection,
        name: &str,
    ) -> Result<(), anyhow::Error> {
        sqlx::query("DO RELEASE_LOCK(?)")
            .bind(sha256::digest(name))
            .execute(&mut *conn)
            .await?;
        Ok(())
    }
}
