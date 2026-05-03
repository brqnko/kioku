#[async_trait::async_trait]
pub trait ChatbotRepository<C>: Send + Sync {
    async fn find_for_update(&self, c: &mut C, id: uuid::Uuid) -> Result<Option<super::domain::Chatbot>, anyhow::Error>;
    async fn save(&self, c: &mut C, chatbot: super::domain::Chatbot) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
    async fn remove(&self, c: &mut C, id: uuid::Uuid) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
}

pub struct ChatbotRepositoryImpl {}

impl ChatbotRepositoryImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl ChatbotRepository<sqlx::MySqlConnection> for ChatbotRepositoryImpl {
    async fn find_for_update(&self, c: &mut sqlx::MySqlConnection, id: uuid::Uuid) -> Result<Option<super::domain::Chatbot>, anyhow::Error> {
        todo!()
    }

    async fn save(&self, c: &mut sqlx::MySqlConnection, chatbot: super::domain::Chatbot) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error> {
        todo!()
    }

    async fn remove(&self, c: &mut sqlx::MySqlConnection, id: uuid::Uuid) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error> {
        todo!()
    }
}
