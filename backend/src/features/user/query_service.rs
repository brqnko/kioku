pub struct UserProfileView {
    pub id: uuid::Uuid,
    pub display_name: String,
    pub language_code: String,
    pub joined_at: chrono::DateTime<chrono::Utc>,
}

pub struct RefreshTokenView {
    pub id: uuid::Uuid,
    pub ip_address: String,
    pub user_agent: String,
    pub activated_at: chrono::DateTime<chrono::Utc>,
    pub last_used_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait::async_trait]
pub trait QueryService: Send + Sync {
    async fn get_user_profile(&self, user_id: uuid::Uuid) -> Result<Option<UserProfileView>, anyhow::Error>;
    async fn list_refresh_tokens_by_user_id(
        &self,
        user_id: uuid::Uuid,
        cursor: Option<uuid::Uuid>,
        limit: u32,
    ) -> Result<Vec<RefreshTokenView>, anyhow::Error>;
}

pub struct QueryServiceImpl {
    pool: sqlx::Pool<sqlx::MySql>
}

impl QueryServiceImpl {
    pub fn new(pool: sqlx::Pool<sqlx::MySql>) -> Self {
        Self {
            pool,
        }
    }
}

#[async_trait::async_trait]
impl QueryService for QueryServiceImpl {
    async fn get_user_profile(&self, user_id: uuid::Uuid) -> Result<Option<UserProfileView>, anyhow::Error> {
        todo!()
    }

    async fn list_refresh_tokens_by_user_id(
        &self,
        user_id: uuid::Uuid,
        cursor: Option<uuid::Uuid>,
        limit: u32,
    ) -> Result<Vec<RefreshTokenView>, anyhow::Error> {
        todo!()
    }
}
