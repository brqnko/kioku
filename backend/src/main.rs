struct Config {
    database_url: String,
    frontend_url: String,
    backend_url: String,
    port: u16,
    mysql_kind: backend::util::dialect::MySQLKind,
    google_oidc_client_id: String,
    google_oidc_client_secret: String,
    s3_endpoint_url: String,
    s3_region: String,
    s3_access_key_id: String,
    s3_secret_access_key: String,
    s3_provider_name: String,
    s3_bucket: String,
    s3_temporary_bucket: String,
    redis_url: String,
    gemini_api_key: String,
    github_token: String,
}

impl Config {
    fn from_env() -> anyhow::Result<Self> {
        fn require(key: &str) -> anyhow::Result<String> {
            use anyhow::Context;
            std::env::var(key).with_context(|| format!("env var not set: {key}"))
        }

        Ok(Self {
            database_url: require("DATABASE_URL")?,
            frontend_url: require("FRONTEND_URL")?,
            backend_url: require("BACKEND_URL")?,
            port: require("PORT")?.parse::<u16>()?,
            mysql_kind: match require("MYSQL_KIND")?.as_str() {
                "mariadb" => backend::util::dialect::MySQLKind::MariaDB,
                "tidb" => backend::util::dialect::MySQLKind::TiDB,
                other => anyhow::bail!("invalid MYSQL_KIND: {other}"),
            },
            google_oidc_client_id: require("GOOGLE_OIDC_CLIENT_ID")?,
            google_oidc_client_secret: require("GOOGLE_OIDC_CLIENT_SECRET")?,
            s3_endpoint_url: require("S3_ENDPOINT_URL")?,
            s3_region: require("S3_REGION")?,
            s3_access_key_id: require("S3_ACCESS_KEY_ID")?,
            s3_secret_access_key: require("S3_SECRET_ACCESS_KEY")?,
            s3_provider_name: require("S3_PROVIDER_NAME")?,
            s3_bucket: require("S3_BUCKET")?,
            s3_temporary_bucket: require("S3_TEMPORARY_BUCKET")?,
            redis_url: require("REDIS_URL")?,
            gemini_api_key: require("GEMINI_API_KEY")?,
            github_token: require("GITHUB_TOKEN")?,
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use std::sync::Arc;

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,tower_http=info")),
        )
        .init();

    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| anyhow::anyhow!("failed to install rustls crypto provider"))?;

    let config = Config::from_env()?;

    let pool = sqlx::mysql::MySqlPoolOptions::new()
        .connect(&config.database_url)
        .await?;

    let oidc_client: Arc<dyn backend::features::user::service::OIDCClient> = Arc::new(
        backend::features::user::service::GoogleOIDCClientImpl::new(
            config.google_oidc_client_id,
            config.google_oidc_client_secret,
            "https://accounts.google.com".to_string(),
            format!("{}/auth/oidc/callback", config.backend_url),
        )
        .await?,
    );

    let jwt_private_key = std::fs::read("/run/secrets/jwt-private")?;
    let jwt_public_key = std::fs::read("/run/secrets/jwt-public")?;
    let jwt_service: Arc<dyn backend::util::jwt::JWTService> =
        Arc::new(backend::util::jwt::JWTServiceImpl::new(
            "kioku".to_string(),
            &jwt_private_key,
            &jwt_public_key,
        )?);

    let storage_service: Arc<dyn backend::util::storage::StorageService> =
        Arc::new(backend::util::storage::StorageServiceImpl::new(
            config.s3_endpoint_url.clone(),
            config.s3_region.clone(),
            config.s3_access_key_id.clone(),
            config.s3_secret_access_key.clone(),
            config.s3_provider_name.clone(),
            config.s3_bucket,
        )?);

    let temporary_storage_impl = backend::util::storage::StorageServiceImpl::new(
        config.s3_endpoint_url,
        config.s3_region,
        config.s3_access_key_id,
        config.s3_secret_access_key,
        config.s3_provider_name,
        config.s3_temporary_bucket,
    )?;
    temporary_storage_impl.set_expiration_days(1).await?;
    temporary_storage_impl
        .set_cors(&config.frontend_url)
        .await?;
    let temporary_storage_service: Arc<dyn backend::util::storage::StorageService> =
        Arc::new(temporary_storage_impl);

    let redis_pool = deadpool_redis::Config::from_url(&config.redis_url)
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))?;

    let jti_blacklist_service: Arc<dyn backend::util::jti_blacklist::JtiBlacklistService> =
        Arc::new(backend::util::jti_blacklist::JtiBlacklistServiceImpl::new(
            redis_pool.clone(),
        ));

    let embedding_client: Arc<dyn backend::util::embedding::EmbeddingClient> =
        Arc::new(backend::util::embedding::EmbeddingClientImpl::new());

    let llm_client: Arc<dyn backend::util::llm::LLMClient> =
        Arc::new(backend::util::llm::CopilotImpl::new(config.github_token)?);

    let tts_client: Arc<dyn backend::util::tts::TTSClient> = Arc::new(
        backend::util::tts::GeminiTtsImpl::new(config.gemini_api_key)?,
    );

    let pdf2md_service: Arc<dyn backend::util::pdf2md::Pdf2MdService> =
        Arc::new(backend::util::pdf2md::Pdf2MdServiceImpl::new());

    let podcast_request_service: Arc<dyn backend::util::podcast_request::PodcastRequestService> =
        Arc::new(
            backend::util::podcast_request::PodcastRequestServiceImpl::new(redis_pool.clone()),
        );

    let app = Arc::new(backend::app::App::new(backend::app::AppArgs {
        pool,
        oidc_client,
        jwt_service,
        storage_service,
        temporary_storage_service,
        jti_blacklist_service,
        embedding_client,
        llm_client,
        tts_client,
        pdf2md_service,
        podcast_request_service,
        mysql_kind: config.mysql_kind,
        access_token_duration: chrono::Duration::hours(1),
        refresh_token_duration: chrono::Duration::days(7),
        frontend_url: config.frontend_url,
    }));

    let bind_addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("listening on {bind_addr}");
    axum::serve(listener, backend::server::router(app))
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }

    tracing::info!("shutdown signal received");
}
