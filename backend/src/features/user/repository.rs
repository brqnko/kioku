#[async_trait::async_trait]
pub trait RefreshTokenRepository<C>: Send + Sync {
    async fn find_for_update(&self, c: &mut C, id: uuid::Uuid) -> Result<Option<super::domain::RefreshToken>, anyhow::Error>;
    async fn find_all_by_user_id_for_update(&self, c: &mut C, user_id: uuid::Uuid) -> Result<Vec<super::domain::RefreshToken>, anyhow::Error>;
    async fn find_by_token_hash_for_update(&self, c: &mut C, token_hash: &str) -> Result<Result<Option<super::domain::RefreshToken>, crate::domain::DomainError>, anyhow::Error>;
    async fn save(&self, c: &mut C, refresh_token: super::domain::RefreshToken) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
    async fn remove(&self, c: &mut C, id: uuid::Uuid) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
    async fn remove_all_by_user_id(&self, c: &mut C, user_id: uuid::Uuid) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
}

pub struct RefreshTokenRepositoryImpl {}

impl RefreshTokenRepositoryImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl RefreshTokenRepository<sqlx::MySqlConnection> for RefreshTokenRepositoryImpl {
    async fn find_for_update(&self, c: &mut sqlx::MySqlConnection, id: uuid::Uuid) -> Result<Option<super::domain::RefreshToken>, anyhow::Error> {
        todo!()
    }

    async fn find_by_token_hash_for_update(&self, c: &mut sqlx::MySqlConnection, token_hash: &str) -> Result<Result<Option<super::domain::RefreshToken>, crate::domain::DomainError>, anyhow::Error> {
        todo!()
    }

    async fn save(&self, c: &mut sqlx::MySqlConnection, refresh_token: super::domain::RefreshToken) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error> {
        todo!()
    }

    async fn find_all_by_user_id_for_update(&self, c: &mut sqlx::MySqlConnection, user_id: uuid::Uuid) -> Result<Vec<super::domain::RefreshToken>, anyhow::Error> {
        todo!()
    }

    async fn remove(&self, c: &mut sqlx::MySqlConnection, id: uuid::Uuid) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error> {
        todo!()
    }

    async fn remove_all_by_user_id(&self, c: &mut sqlx::MySqlConnection, user_id: uuid::Uuid) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error> {
        todo!()
    }
}

#[async_trait::async_trait]
pub trait UserRepository<C>: Send + Sync {
    async fn find_for_update(&self, c: &mut C, id: uuid::Uuid) -> Result<Option<super::domain::User>, anyhow::Error>;
    async fn find_by_iss_sub_for_update(&self, c: &mut C, iss: &str, sub: &str) -> Result<Result<Option<super::domain::User>, crate::domain::DomainError>, anyhow::Error>;
    async fn save(&self, c: &mut C, user: super::domain::User) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
    async fn remove(&self, c: &mut C, id: uuid::Uuid) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
}

pub struct UserRepositoryImpl {
}

impl UserRepositoryImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl UserRepository<sqlx::MySqlConnection> for UserRepositoryImpl {
    async fn find_for_update(&self, c: &mut sqlx::MySqlConnection, id: uuid::Uuid) -> Result<Option<super::domain::User>, anyhow::Error> {
        todo!()
    }

    async fn find_by_iss_sub_for_update(&self, c: &mut sqlx::MySqlConnection, iss: &str, sub: &str) -> Result<Result<Option<super::domain::User>, crate::domain::DomainError>, anyhow::Error> {
        todo!()
    }

    async fn save(&self, c: &mut sqlx::MySqlConnection, user: super::domain::User) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error> {
        todo!()
    }

    async fn remove(&self, c: &mut sqlx::MySqlConnection, id: uuid::Uuid) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error> {
        todo!()
    }
}
