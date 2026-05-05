#[async_trait::async_trait]
pub trait EmbeddingClient: Send + Sync {
    async fn embed(
        &self,
        inputs: std::collections::HashMap<uuid::Uuid, String>,
    ) -> Result<std::collections::HashMap<uuid::Uuid, Vec<f32>>, anyhow::Error>;
}

pub struct EmbeddingClientImpl {
    http_client: reqwest::Client,
    endpoint: String,
}

impl EmbeddingClientImpl {
    pub fn new(endpoint: String) -> Result<Self, anyhow::Error> {
        let http_client = reqwest::Client::builder().build()?;
        Ok(Self {
            http_client,
            endpoint,
        })
    }
}

#[async_trait::async_trait]
impl EmbeddingClient for EmbeddingClientImpl {
    async fn embed(
        &self,
        inputs: std::collections::HashMap<uuid::Uuid, String>,
    ) -> Result<std::collections::HashMap<uuid::Uuid, Vec<f32>>, anyhow::Error> {
        todo!()
    }
}
