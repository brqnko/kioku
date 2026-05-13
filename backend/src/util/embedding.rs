#[async_trait::async_trait]
pub trait EmbeddingClient: Send + Sync {
    async fn embed(
        &self,
        inputs: std::collections::HashMap<uuid::Uuid, String>,
    ) -> Result<std::collections::HashMap<uuid::Uuid, Vec<f32>>, anyhow::Error>;
}

#[derive(Default)]
pub struct EmbeddingClientImpl;

impl EmbeddingClientImpl {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl EmbeddingClient for EmbeddingClientImpl {
    async fn embed(
        &self,
        _inputs: std::collections::HashMap<uuid::Uuid, String>,
    ) -> Result<std::collections::HashMap<uuid::Uuid, Vec<f32>>, anyhow::Error> {
        todo!()
    }
}