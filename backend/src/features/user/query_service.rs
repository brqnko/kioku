#[async_trait::async_trait]
pub trait QueryService {}

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
}
