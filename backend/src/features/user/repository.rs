#[async_trait::async_trait]
pub trait Repository<C>: Send + Sync {
    async fn find_for_update(&self, c: &mut C, id: uuid::Uuid) -> Result<Result<super::domain::User, crate::domain::DomainError>, anyhow::Error>;
    async fn save(&self, c: &mut C, user: super::domain::User) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
    async fn remove(&self, c: &mut C, id: uuid::Uuid) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
}

pub struct RepositoryImpl {
}

impl RepositoryImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl Repository<sqlx::MySqlConnection> for RepositoryImpl {
    async fn find_for_update(&self, c: &mut sqlx::MySqlConnection, id: uuid::Uuid) -> Result<Result<super::domain::User, crate::domain::DomainError>, anyhow::Error> {
        todo!()
    }

    async fn save(&self, c: &mut sqlx::MySqlConnection, user: super::domain::User) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error> {
        todo!()
    }

    async fn remove(&self, c: &mut sqlx::MySqlConnection, id: uuid::Uuid) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error> {
        todo!()
    }
}
