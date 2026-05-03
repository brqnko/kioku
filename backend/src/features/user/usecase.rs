// verify access token

pub struct VerifyAccessTokenInput {
    pub access_token: String,
}

pub struct VerifyAccessTokenOutput {
    pub user_id: uuid::Uuid,
    pub jti: uuid::Uuid,
}

pub struct VerifyAccessTokenUsecaseImpl {
    jwt_service: std::sync::Arc<dyn crate::util::jwt::JWTService>,
    jti_blacklist_service: std::sync::Arc<dyn crate::util::jti_blacklist::JtiBlacklistService>,
}

impl VerifyAccessTokenUsecaseImpl {
    pub fn new(app: &crate::app::App) -> Self {
        Self {
            jwt_service: app.jwt_service.clone(),
            jti_blacklist_service: app.jti_blacklist_service.clone(),
        }
    }
}

#[async_trait::async_trait]
impl crate::usecase::Usecase<VerifyAccessTokenInput, VerifyAccessTokenOutput> for VerifyAccessTokenUsecaseImpl {
    async fn handle(&self, input: VerifyAccessTokenInput) -> Result<Result<VerifyAccessTokenOutput, crate::domain::DomainError>, anyhow::Error> {
        let verified = match self.jwt_service.verify_user_access_token(input.access_token) {
            Ok(ok) => ok,
            Err(_) => {
                return Ok(Err(crate::domain::DomainError::new(
                    "invalid_access_token",
                    "access token is invalid".to_string(),
                    crate::domain::DomainErrorKind::Forbidden,
                )))
            }
        };

        if self.jti_blacklist_service.is_blacklisted(verified.jti).await? {
            return Ok(Err(crate::domain::DomainError::new(
                "revoked_access_token",
                "access token has been revoked".to_string(),
                crate::domain::DomainErrorKind::Forbidden,
            )));
        }

        Ok(Ok(VerifyAccessTokenOutput {
            user_id: verified.user_id,
            jti: verified.jti,
        }))
    }
}

// oidc start

use std::ops::Add;

pub struct OIDCStartInput {}

pub struct OIDCStartOutput {
    pub state_token: String,
    pub redirect_url: String,
    pub nonce: String,
}

pub struct OIDCStartUsecaseImpl {
    oidc_client: std::sync::Arc<dyn super::service::OIDCClient>,
}

impl OIDCStartUsecaseImpl {
    pub fn new(app: &crate::app::App) -> Self {
        Self {
            oidc_client: app.oidc_client.clone(),
        }
    }
}

#[async_trait::async_trait]
impl crate::usecase::Usecase<OIDCStartInput, OIDCStartOutput> for OIDCStartUsecaseImpl {
    async fn handle(&self, input: OIDCStartInput) -> Result<Result<OIDCStartOutput, crate::domain::DomainError>, anyhow::Error> {
        let code_url = self.oidc_client.code_url().await?;

        Ok(Ok(OIDCStartOutput {
            state_token: code_url.csrf_state,
            redirect_url: code_url.url,
            nonce: code_url.nonce,
        }))
    }
}

// oidc callback

pub struct OIDCCallbackInput {
    pub code: String,
    pub state: String,
    pub expected_state: String,
    pub nonce: String,
    pub display_name: String,
    pub language_code: String,
    pub ip_address: String,
    pub user_agent: String,
}

pub struct OIDCCallbackOutput {
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expires_at: chrono::DateTime<chrono::Utc>,
    pub csrf: String,
    pub refresh_token_expires_at: chrono::DateTime<chrono::Utc>,
    pub redirect_to: String,
}

pub struct OIDCCallbackUsecaseImpl {
    oidc_client: std::sync::Arc<dyn super::service::OIDCClient>,
    pool: sqlx::Pool<sqlx::MySql>,
    user_repository: Box<dyn super::repository::UserRepository<sqlx::MySqlConnection>>,
    refresh_token_repository: Box<dyn super::repository::RefreshTokenRepository<sqlx::MySqlConnection>>,
    jwt_service: std::sync::Arc<dyn crate::util::jwt::JWTService>,
    access_token_duration: chrono::Duration,
    refresh_token_duration: chrono::Duration,
    frontend_url: String,
}

impl OIDCCallbackUsecaseImpl {
    pub fn new(app: &crate::app::App) -> Self {
        Self {
            oidc_client: app.oidc_client.clone(),
            pool: app.pool.clone(),
            user_repository: Box::new(super::repository::UserRepositoryImpl::new()),
            refresh_token_repository: Box::new(super::repository::RefreshTokenRepositoryImpl::new()),
            jwt_service: app.jwt_service.clone(),
            access_token_duration: app.access_token_duration,
            refresh_token_duration: app.refresh_token_duration,
            frontend_url: app.frontend_url.clone(),
        }
    }
}

#[async_trait::async_trait]
impl crate::usecase::Usecase<OIDCCallbackInput, OIDCCallbackOutput> for OIDCCallbackUsecaseImpl {
    async fn handle(&self, input: OIDCCallbackInput) -> Result<Result<OIDCCallbackOutput, crate::domain::DomainError>, anyhow::Error> {
        if input.state != input.expected_state {
            return Ok(Err(crate::domain::DomainError::new(
                "invalid_state",
                "CSRF state mismatch".to_string(),
                crate::domain::DomainErrorKind::BadInput,
            )));
        }

        let result = self.oidc_client.exchange_and_verify(input.code, input.nonce).await?;

        let mut tx = self.pool.begin().await?;

        let (user, is_new_user) = match self.user_repository.find_by_iss_sub_for_update(&mut tx, &result.iss, &result.sub).await? {
            Ok(Some(ok)) => (ok, false),
            Ok(None) => match super::domain::User::new(
                input.display_name,
                input.language_code,
                result.iss,
                result.sub,
                super::domain::UserOption {
                    ..Default::default()
                },
            )? {
                Ok(ok) => (ok, true),
                Err(err) => {
                    return Ok(Err(err))
                }
            },
            Err(err) => {
                return Ok(Err(err))
            }
        };

        let mut refresh_token = match super::domain::RefreshToken::new(
            user.id,
            input.ip_address,
            input.user_agent,
            super::domain::RefreshTokenOption {
                ..Default::default()
            },
        )? {
            Ok(ok) => ok,
            Err(err) => {
                return Ok(Err(err));
            }
        };

        let access_token_jti = uuid::Uuid::new_v4();
        let (refresh_token_raw, refresh_token_expires_at) = refresh_token.rotate(self.refresh_token_duration, access_token_jti)?;

        let (access_token, access_token_expires_at) = self.jwt_service.sign_user_access_token(user.id, access_token_jti, self.access_token_duration)?;

        if is_new_user {
            match self.user_repository.save(&mut tx, user).await? {
                Ok(ok) => ok,
                Err(err) => {
                    return Ok(Err(err))
                }
            };
        }

        let _ = match self.refresh_token_repository.save(&mut tx, refresh_token).await? {
            Ok(ok) => ok,
            Err(err) => {
                return Ok(Err(err));
            }
        };

        tx.commit().await?;

        let csrf = crate::util::random::random_string(32);

        Ok(Ok(OIDCCallbackOutput {
            access_token,
            refresh_token: refresh_token_raw,
            access_token_expires_at,
            csrf,
            refresh_token_expires_at,
            redirect_to: format!("{}/dashboard", self.frontend_url),
        }))
    }
}

// logout

pub struct LogoutInput {
    pub refresh_token: String,
}

pub struct LogoutOutput {}

pub struct LogoutUsecaseImpl {
    pool: sqlx::Pool<sqlx::MySql>,
    refresh_token_repository: Box<dyn super::repository::RefreshTokenRepository<sqlx::MySqlConnection>>,
    jti_blacklist_service: std::sync::Arc<dyn crate::util::jti_blacklist::JtiBlacklistService>,
    access_token_duration: chrono::Duration,
}

impl LogoutUsecaseImpl {
    pub fn new(app: &crate::app::App) -> Self {
        Self {
            pool: app.pool.clone(),
            refresh_token_repository: Box::new(super::repository::RefreshTokenRepositoryImpl::new()),
            jti_blacklist_service: app.jti_blacklist_service.clone(),
            access_token_duration: app.access_token_duration,
        }
    }
}

#[async_trait::async_trait]
impl crate::usecase::Usecase<LogoutInput, LogoutOutput> for LogoutUsecaseImpl {
    async fn handle(&self, input: LogoutInput) -> Result<Result<LogoutOutput, crate::domain::DomainError>, anyhow::Error> {
        let mut tx = self.pool.begin().await?;

        let refresh_token = match self.refresh_token_repository.find_by_token_hash_for_update(&mut tx, &sha256::digest(&input.refresh_token)).await? {
            Ok(Some(ok)) => ok,
            Ok(None) => {
                return Ok(Err(crate::domain::DomainError::new(
                    "invalid_refresh_token",
                    "refresh token not found".to_string(),
                    crate::domain::DomainErrorKind::NotFound,
                )))
            }
            Err(err) => {
                return Ok(Err(err))
            }
        };

        self.jti_blacklist_service.add(refresh_token.access_token_jti, chrono::Utc::now().add(self.access_token_duration)).await?;

        match self.refresh_token_repository.remove(&mut tx, refresh_token.id).await? {
            Ok(ok) => ok,
            Err(err) => {
                return Ok(Err(err))
            }
        };

        tx.commit().await?;

        Ok(Ok(LogoutOutput {  }))
    }
}

// refresh

pub struct RefreshInput {
    pub refresh_token: String,
    pub ip_address: String,
    pub user_agent: String,
}

pub struct RefreshOutput {
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expires_at: chrono::DateTime<chrono::Utc>,
    pub refresh_token_expires_at: chrono::DateTime<chrono::Utc>,
}

pub struct RefreshUsecaseImpl {
    pool: sqlx::Pool<sqlx::MySql>,
    refresh_token_repository: Box<dyn super::repository::RefreshTokenRepository<sqlx::MySqlConnection>>,
    jwt_service: std::sync::Arc<dyn crate::util::jwt::JWTService>,
    access_token_duration: chrono::Duration,
    refresh_token_duration: chrono::Duration,
}

impl RefreshUsecaseImpl {
    pub fn new(app: &crate::app::App) -> Self {
        Self {
            pool: app.pool.clone(),
            refresh_token_repository: Box::new(super::repository::RefreshTokenRepositoryImpl::new()),
            jwt_service: app.jwt_service.clone(),
            access_token_duration: app.access_token_duration,
            refresh_token_duration: app.refresh_token_duration,
        }
    }
}

#[async_trait::async_trait]
impl crate::usecase::Usecase<RefreshInput, RefreshOutput> for RefreshUsecaseImpl {
    async fn handle(&self, input: RefreshInput) -> Result<Result<RefreshOutput, crate::domain::DomainError>, anyhow::Error> {
        let mut tx = self.pool.begin().await?;

        let mut refresh_token = match self.refresh_token_repository.find_by_token_hash_for_update(&mut tx, &sha256::digest(&input.refresh_token)).await? {
            Ok(Some(ok)) => ok,
            Ok(None) => {
                return Ok(Err(crate::domain::DomainError::new(
                    "invalid_refresh_token",
                    "refresh token not found".to_string(),
                    crate::domain::DomainErrorKind::NotFound,
                )))
            }
            Err(err) => {
                return Ok(Err(err))
            }
        };

        if chrono::Utc::now() < refresh_token.last_used_at + self.access_token_duration {
            return Ok(Err(crate::domain::DomainError::new(
                "access_token_still_valid",
                "access token has not expired yet".to_string(),
                crate::domain::DomainErrorKind::BadInput,
            )));
        }

        let access_token_jti = uuid::Uuid::new_v4();
        let (refresh_token_raw, refresh_token_expires_at) = refresh_token.rotate(self.refresh_token_duration, access_token_jti)?;
        let (access_token, access_token_expires_at) = self.jwt_service.sign_user_access_token(refresh_token.user_id, access_token_jti, self.access_token_duration)?;

        match self.refresh_token_repository.save(&mut tx, refresh_token).await? {
            Ok(ok) => ok,
            Err(err) => {
                return Ok(Err(err))
            }
        }

        tx.commit().await?;

        Ok(Ok(RefreshOutput {
            access_token,
            refresh_token: refresh_token_raw,
            access_token_expires_at,
            refresh_token_expires_at,
        }))
    }
}

// get user profile

pub struct GetUserProfileInput {
    pub user_id: uuid::Uuid,
}

pub struct GetUserProfileOutput {
    pub profile: super::query_service::UserProfileView,
}

pub struct GetUserProfileUsecaseImpl {
    query_service: super::query_service::QueryServiceImpl,
}

impl GetUserProfileUsecaseImpl {
    pub fn new(app: &crate::app::App) -> Self {
        Self {
            query_service: super::query_service::QueryServiceImpl::new(app.pool.clone()),
        }
    }
}

#[async_trait::async_trait]
impl crate::usecase::Usecase<GetUserProfileInput, GetUserProfileOutput> for GetUserProfileUsecaseImpl {
    async fn handle(&self, input: GetUserProfileInput) -> Result<Result<GetUserProfileOutput, crate::domain::DomainError>, anyhow::Error> {
        let profile = match self.query_service.get_user_profile(input.user_id).await? {
            Some(ok) => ok,
            None => {
                return Ok(Err(crate::domain::DomainError::new(
                    "user_not_found",
                    "user not found".to_string(),
                    crate::domain::DomainErrorKind::NotFound,
                )))
            }
        };

        Ok(Ok(GetUserProfileOutput { profile }))
    }
}

// update user profile

pub struct UpdateUserProfileInput {
    pub user_id: uuid::Uuid,
    pub display_name: Option<String>,
    pub language_code: Option<String>,
}

pub struct UpdateUserProfileOutput {}

pub struct UpdateUserProfileUsecaseImpl {
    pool: sqlx::Pool<sqlx::MySql>,
    user_repository: Box<dyn super::repository::UserRepository<sqlx::MySqlConnection>>,
}

impl UpdateUserProfileUsecaseImpl {
    pub fn new(app: &crate::app::App) -> Self {
        Self {
            pool: app.pool.clone(),
            user_repository: Box::new(super::repository::UserRepositoryImpl::new()),
        }
    }
}

#[async_trait::async_trait]
impl crate::usecase::Usecase<UpdateUserProfileInput, UpdateUserProfileOutput> for UpdateUserProfileUsecaseImpl {
    async fn handle(&self, input: UpdateUserProfileInput) -> Result<Result<UpdateUserProfileOutput, crate::domain::DomainError>, anyhow::Error> {
        let mut tx = self.pool.begin().await?;

        let mut user = match self.user_repository.find_for_update(&mut tx, input.user_id).await? {
            Some(ok) => ok,
            None => {
                return Ok(Err(crate::domain::DomainError::new(
                    "user_not_found",
                    "user not found".to_string(),
                    crate::domain::DomainErrorKind::NotFound,
                )))
            }
        };

        if let Some(display_name) = input.display_name {
            match user.set_display_name(display_name) {
                Ok(ok) => ok,
                Err(err) => {
                    return Ok(Err(err))
                }
            }
        }

        if let Some(language_code) = input.language_code {
            match user.set_language_code(language_code) {
                Ok(ok) => ok,
                Err(err) => {
                    return Ok(Err(err))
                }
            }
        }

        match self.user_repository.save(&mut tx, user).await? {
            Ok(ok) => ok,
            Err(err) => {
                return Ok(Err(err))
            }
        }

        tx.commit().await?;

        Ok(Ok(UpdateUserProfileOutput {}))
    }
}

// list refresh tokens

pub struct ListRefreshTokensInput {
    pub user_id: uuid::Uuid,
    pub cursor: Option<uuid::Uuid>,
    pub limit: u32,
}

pub struct ListRefreshTokensOutput {
    pub items: Vec<super::query_service::RefreshTokenView>,
    pub next_cursor: Option<uuid::Uuid>,
}

pub struct ListRefreshTokensUsecaseImpl {
    query_service: super::query_service::QueryServiceImpl,
}

impl ListRefreshTokensUsecaseImpl {
    pub fn new(app: &crate::app::App) -> Self {
        Self {
            query_service: super::query_service::QueryServiceImpl::new(app.pool.clone()),
        }
    }
}

#[async_trait::async_trait]
impl crate::usecase::Usecase<ListRefreshTokensInput, ListRefreshTokensOutput> for ListRefreshTokensUsecaseImpl {
    async fn handle(&self, input: ListRefreshTokensInput) -> Result<Result<ListRefreshTokensOutput, crate::domain::DomainError>, anyhow::Error> {
        if input.limit > 32 {
            return Ok(Err(crate::domain::DomainError::new(
                "invalid_limit",
                "limit must be 32 or less".to_string(),
                crate::domain::DomainErrorKind::BadInput,
            )));
        }

        let mut rows = self.query_service.list_refresh_tokens_by_user_id(
            input.user_id,
            input.cursor,
            input.limit + 1,
        ).await?;

        let next_cursor = if rows.len() as u32 > input.limit {
            rows.pop().map(|r| r.id)
        } else {
            None
        };

        Ok(Ok(ListRefreshTokensOutput { items: rows, next_cursor }))
    }
}

// revoke refresh token

pub struct RevokeRefreshTokenInput {
    pub user_id: uuid::Uuid,
    pub refresh_token_id: uuid::Uuid,
}

pub struct RevokeRefreshTokenOutput {}

pub struct RevokeRefreshTokenUsecaseImpl {
    pool: sqlx::Pool<sqlx::MySql>,
    refresh_token_repository: Box<dyn super::repository::RefreshTokenRepository<sqlx::MySqlConnection>>,
    jti_blacklist_service: std::sync::Arc<dyn crate::util::jti_blacklist::JtiBlacklistService>,
    access_token_duration: chrono::Duration,
}

impl RevokeRefreshTokenUsecaseImpl {
    pub fn new(app: &crate::app::App) -> Self {
        Self {
            pool: app.pool.clone(),
            refresh_token_repository: Box::new(super::repository::RefreshTokenRepositoryImpl::new()),
            jti_blacklist_service: app.jti_blacklist_service.clone(),
            access_token_duration: app.access_token_duration,
        }
    }
}

#[async_trait::async_trait]
impl crate::usecase::Usecase<RevokeRefreshTokenInput, RevokeRefreshTokenOutput> for RevokeRefreshTokenUsecaseImpl {
    async fn handle(&self, input: RevokeRefreshTokenInput) -> Result<Result<RevokeRefreshTokenOutput, crate::domain::DomainError>, anyhow::Error> {
        let mut tx = self.pool.begin().await?;

        let refresh_token = match self.refresh_token_repository.find_for_update(&mut tx, input.refresh_token_id).await? {
            Some(ok) => ok,
            None => {
                return Ok(Err(crate::domain::DomainError::new(
                    "refresh_token_not_found",
                    "refresh token not found".to_string(),
                    crate::domain::DomainErrorKind::NotFound,
                )))
            }
        };

        if refresh_token.user_id != input.user_id {
            return Ok(Err(crate::domain::DomainError::new(
                "forbidden",
                "refresh token does not belong to the user".to_string(),
                crate::domain::DomainErrorKind::BadInput,
            )));
        }

        self.jti_blacklist_service.add(refresh_token.access_token_jti, chrono::Utc::now().add(self.access_token_duration)).await?;

        match self.refresh_token_repository.remove(&mut tx, refresh_token.id).await? {
            Ok(ok) => ok,
            Err(err) => {
                return Ok(Err(err))
            }
        }

        tx.commit().await?;

        Ok(Ok(RevokeRefreshTokenOutput {}))
    }
}

// remove user

pub struct RemoveUserInput {
    pub user_id: uuid::Uuid,
}

pub struct RemoveUserOutput {}

pub struct RemoveUserUsecaseImpl {
    pool: sqlx::Pool<sqlx::MySql>,
    user_repository: Box<dyn super::repository::UserRepository<sqlx::MySqlConnection>>,
}

impl RemoveUserUsecaseImpl {
    pub fn new(app: &crate::app::App) -> Self {
        Self {
            pool: app.pool.clone(),
            user_repository: Box::new(super::repository::UserRepositoryImpl::new()),
        }
    }
}

#[async_trait::async_trait]
impl crate::usecase::Usecase<RemoveUserInput, RemoveUserOutput> for RemoveUserUsecaseImpl {
    async fn handle(&self, input: RemoveUserInput) -> Result<Result<RemoveUserOutput, crate::domain::DomainError>, anyhow::Error> {
        let mut tx = self.pool.begin().await?;

        match self.user_repository.find_for_update(&mut tx, input.user_id).await? {
            Some(_) => {}
            None => {
                return Ok(Err(crate::domain::DomainError::new(
                    "user_not_found",
                    "user not found".to_string(),
                    crate::domain::DomainErrorKind::NotFound,
                )))
            }
        }

        match self.user_repository.remove(&mut tx, input.user_id).await? {
            Ok(_) => {}
            Err(err) => {
                return Ok(Err(err))
            }
        }

        tx.commit().await?;

        Ok(Ok(RemoveUserOutput {}))
    }
}

// revoke all refresh tokens

pub struct RevokeAllRefreshTokensInput {
    pub user_id: uuid::Uuid,
}

pub struct RevokeAllRefreshTokensOutput {}

pub struct RevokeAllRefreshTokensUsecaseImpl {
    pool: sqlx::Pool<sqlx::MySql>,
    refresh_token_repository: Box<dyn super::repository::RefreshTokenRepository<sqlx::MySqlConnection>>,
    jti_blacklist_service: std::sync::Arc<dyn crate::util::jti_blacklist::JtiBlacklistService>,
    access_token_duration: chrono::Duration,
}

impl RevokeAllRefreshTokensUsecaseImpl {
    pub fn new(app: &crate::app::App) -> Self {
        Self {
            pool: app.pool.clone(),
            refresh_token_repository: Box::new(super::repository::RefreshTokenRepositoryImpl::new()),
            jti_blacklist_service: app.jti_blacklist_service.clone(),
            access_token_duration: app.access_token_duration,
        }
    }
}

#[async_trait::async_trait]
impl crate::usecase::Usecase<RevokeAllRefreshTokensInput, RevokeAllRefreshTokensOutput> for RevokeAllRefreshTokensUsecaseImpl {
    async fn handle(&self, input: RevokeAllRefreshTokensInput) -> Result<Result<RevokeAllRefreshTokensOutput, crate::domain::DomainError>, anyhow::Error> {
        let mut tx = self.pool.begin().await?;

        let tokens = self.refresh_token_repository.find_all_by_user_id_for_update(&mut tx, input.user_id).await?;

        let expires_at = chrono::Utc::now().add(self.access_token_duration);
        for token in &tokens {
            self.jti_blacklist_service.add(token.access_token_jti, expires_at).await?;
        }

        match self.refresh_token_repository.remove_all_by_user_id(&mut tx, input.user_id).await? {
            Ok(_) => {}
            Err(err) => {
                return Ok(Err(err))
            }
        }

        tx.commit().await?;

        Ok(Ok(RevokeAllRefreshTokensOutput {}))
    }
}

#[cfg(test)]
mod tests {
}