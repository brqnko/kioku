#[async_trait::async_trait]
pub trait Usecase<I, O>: Sized {
    async fn handle(&self, input: I) -> Result<Result<O, crate::domain::DomainError>, anyhow::Error>;
}
