#[async_trait::async_trait]
pub trait ProjectRepository<C>: Send + Sync {
    async fn find_for_update(&self, c: &mut C, id: uuid::Uuid) -> Result<Option<super::domain::Project>, anyhow::Error>;
    async fn save(&self, c: &mut C, project: super::domain::Project) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
    async fn remove(&self, c: &mut C, id: uuid::Uuid) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
}

pub struct ProjectRepositoryImpl {}

impl ProjectRepositoryImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl ProjectRepository<sqlx::MySqlConnection> for ProjectRepositoryImpl {
    async fn find_for_update(&self, c: &mut sqlx::MySqlConnection, id: uuid::Uuid) -> Result<Option<super::domain::Project>, anyhow::Error> {
        todo!()
    }

    async fn save(&self, c: &mut sqlx::MySqlConnection, project: super::domain::Project) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error> {
        todo!()
    }

    async fn remove(&self, c: &mut sqlx::MySqlConnection, id: uuid::Uuid) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error> {
        todo!()
    }
}
