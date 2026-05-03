pub struct App {
    pub pool: sqlx::Pool<sqlx::MySql>,
    pub oidc_client: std::sync::Arc<dyn crate::features::user::service::OIDCClient>,
}
