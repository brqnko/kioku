pub struct App {
    pub pool: sqlx::Pool<sqlx::MySql>,
    pub oidc_client: std::sync::Arc<dyn crate::features::user::service::OIDCClient>,
    pub jwt_service: std::sync::Arc<dyn crate::util::jwt::JWTService>,
    pub jti_blacklist_service: std::sync::Arc<dyn crate::util::jti_blacklist::JtiBlacklistService>,
    pub access_token_duration: chrono::Duration,
    pub refresh_token_duration: chrono::Duration,
    pub frontend_url: String,
}
