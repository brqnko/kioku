// oidc start

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
}

pub struct OIDCCallbackOutput {}

pub struct OIDCCallbackUsecaseImpl {
    oidc_client: std::sync::Arc<dyn super::service::OIDCClient>,
    pool: sqlx::Pool<sqlx::MySql>,
    user_repository: Box<dyn super::repository::Repository<sqlx::MySqlConnection>>,
}

impl OIDCCallbackUsecaseImpl {
    pub fn new(app: &crate::app::App) -> Self {
        Self {
            oidc_client: app.oidc_client.clone(),
            pool: app.pool.clone(),
            user_repository: Box::new(super::repository::RepositoryImpl::new()),
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

        let user = match super::domain::User::new(
            input.display_name,
            input.language_code,
            result.iss,
            result.sub,
            super::domain::UserOption {
                ..Default::default()
            },
        )? {
            Ok(ok) => ok,
            Err(err) => {
                return Ok(Err(err))
            }
        };

        let mut tx = self.pool.begin().await?;

        match self.user_repository.save(&mut tx, user).await? {
            Ok(ok) => ok,
            Err(err) => {
                return Ok(Err(err))
            }
        }

        // TODO: refresh tokenの発行

        tx.commit().await?;

        todo!()
    }
}

// login

pub struct LoginInput {
    pub user_id: uuid::Uuid,
    pub ip_address: String,
    pub user_agent: String,
}

pub struct LoginOutput {
    pub access_token: String,
    pub refresh_token: String,
}

pub struct LoginUsecaseImpl {
    pool: sqlx::Pool<sqlx::MySql>,
}

impl LoginUsecaseImpl {
    pub fn new(app: &crate::app::App) -> Self {
        Self {
            pool: app.pool.clone(),
        }
    }
}

#[async_trait::async_trait]
impl crate::usecase::Usecase<LoginInput, LoginOutput> for LoginUsecaseImpl {
    async fn handle(&self, input: LoginInput) -> Result<Result<LoginOutput, crate::domain::DomainError>, anyhow::Error> {
        todo!()
    }
}

// logout

pub struct LogoutInput {
    pub refresh_token: String,
}

pub struct LogoutOutput {}

pub struct LogoutUsecaseImpl {
    pool: sqlx::Pool<sqlx::MySql>,
}

impl LogoutUsecaseImpl {
    pub fn new(app: &crate::app::App) -> Self {
        Self {
            pool: app.pool.clone(),
        }
    }
}

#[async_trait::async_trait]
impl crate::usecase::Usecase<LogoutInput, LogoutOutput> for LogoutUsecaseImpl {
    async fn handle(&self, input: LogoutInput) -> Result<Result<LogoutOutput, crate::domain::DomainError>, anyhow::Error> {
        todo!()
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
}

pub struct RefreshUsecaseImpl {
    pool: sqlx::Pool<sqlx::MySql>,
}

impl RefreshUsecaseImpl {
    pub fn new(app: &crate::app::App) -> Self {
        Self {
            pool: app.pool.clone(),
        }
    }
}

#[async_trait::async_trait]
impl crate::usecase::Usecase<RefreshInput, RefreshOutput> for RefreshUsecaseImpl {
    async fn handle(&self, input: RefreshInput) -> Result<Result<RefreshOutput, crate::domain::DomainError>, anyhow::Error> {
        todo!()
    }
}
