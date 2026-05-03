#[async_trait::async_trait]
pub trait PodcastRepository<C>: Send + Sync {
    async fn find_for_update(&self, c: &mut C, id: uuid::Uuid) -> Result<Option<super::domain::Podcast>, anyhow::Error>;
    async fn save(&self, c: &mut C, podcast: super::domain::Podcast) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
    async fn remove(&self, c: &mut C, id: uuid::Uuid) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
}

pub struct PodcastRepositoryImpl {}

impl PodcastRepositoryImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl PodcastRepository<sqlx::MySqlConnection> for PodcastRepositoryImpl {
    async fn find_for_update(&self, c: &mut sqlx::MySqlConnection, id: uuid::Uuid) -> Result<Option<super::domain::Podcast>, anyhow::Error> {
        todo!()
    }

    async fn save(&self, c: &mut sqlx::MySqlConnection, podcast: super::domain::Podcast) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error> {
        todo!()
    }

    async fn remove(&self, c: &mut sqlx::MySqlConnection, id: uuid::Uuid) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error> {
        todo!()
    }
}
