pub struct App {
    pub pool: sqlx::Pool<sqlx::MySql>,
    pub oidc_client: std::sync::Arc<dyn crate::features::user::service::OIDCClient>,
    pub jwt_service: std::sync::Arc<dyn crate::util::jwt::JWTService>,
    pub storage_service: std::sync::Arc<dyn crate::util::storage::StorageService>,
    pub temporary_storage_service: std::sync::Arc<dyn crate::util::storage::StorageService>,
    pub jti_blacklist_service: std::sync::Arc<dyn crate::util::jti_blacklist::JtiBlacklistService>,
    pub access_token_duration: chrono::Duration,
    pub refresh_token_duration: chrono::Duration,
    pub frontend_url: String,
    pub user_repository: std::sync::Arc<
        dyn crate::features::user::repository::UserRepository<sqlx::MySqlConnection>,
    >,
    pub refresh_token_repository: std::sync::Arc<
        dyn crate::features::user::repository::RefreshTokenRepository<sqlx::MySqlConnection>,
    >,
    pub user_query_service: std::sync::Arc<dyn crate::features::user::query_service::QueryService>,
    pub project_repository: std::sync::Arc<
        dyn crate::features::project::repository::ProjectRepository<sqlx::MySqlConnection>,
    >,
    pub project_query_service:
        std::sync::Arc<dyn crate::features::project::query_service::QueryService>,
    pub file_repository: std::sync::Arc<
        dyn crate::features::file::repository::FileRepository<sqlx::MySqlConnection>,
    >,
    pub folder_repository: std::sync::Arc<
        dyn crate::features::file::repository::FolderRepository<sqlx::MySqlConnection>,
    >,
    pub text_storage_repository: std::sync::Arc<
        dyn crate::features::file::repository::TextStorageRepository<sqlx::MySqlConnection>,
    >,
    pub file_query_service: std::sync::Arc<dyn crate::features::file::query_service::QueryService>,
    pub file_embedding_repository: std::sync::Arc<
        dyn crate::features::file::repository::FileEmbeddingRepository<sqlx::MySqlConnection>,
    >,
    pub embedding_client: std::sync::Arc<dyn crate::util::embedding::EmbeddingClient>,
    pub queue_service: std::sync::Arc<dyn crate::util::queue::QueueService>,
}

impl App {
    pub fn new(
        pool: sqlx::Pool<sqlx::MySql>,
        oidc_client: std::sync::Arc<dyn crate::features::user::service::OIDCClient>,
        jwt_service: std::sync::Arc<dyn crate::util::jwt::JWTService>,
        storage_service: std::sync::Arc<dyn crate::util::storage::StorageService>,
        temporary_storage_service: std::sync::Arc<dyn crate::util::storage::StorageService>,
        jti_blacklist_service: std::sync::Arc<dyn crate::util::jti_blacklist::JtiBlacklistService>,
        embedding_client: std::sync::Arc<dyn crate::util::embedding::EmbeddingClient>,
        queue_service: std::sync::Arc<dyn crate::util::queue::QueueService>,
        mysql_kind: crate::util::dialect::MySQLKind,
        access_token_duration: chrono::Duration,
        refresh_token_duration: chrono::Duration,
        frontend_url: String,
    ) -> Self {
        use std::sync::Arc;

        Self {
            user_repository: Arc::new(crate::features::user::repository::UserRepositoryImpl::new()),
            refresh_token_repository: Arc::new(
                crate::features::user::repository::RefreshTokenRepositoryImpl::new(),
            ),
            user_query_service: Arc::new(
                crate::features::user::query_service::QueryServiceImpl::new(pool.clone()),
            ),
            project_repository: Arc::new(
                crate::features::project::repository::ProjectRepositoryImpl::new(),
            ),
            project_query_service: Arc::new(
                crate::features::project::query_service::QueryServiceImpl::new(pool.clone()),
            ),
            file_repository: Arc::new(crate::features::file::repository::FileRepositoryImpl::new()),
            folder_repository: Arc::new(
                crate::features::file::repository::FolderRepositoryImpl::new(),
            ),
            text_storage_repository: Arc::new(
                crate::features::file::repository::TextStorageRepositoryImpl::new(),
            ),
            file_query_service: Arc::new(
                crate::features::file::query_service::QueryServiceImpl::new(pool.clone()),
            ),
            file_embedding_repository: Arc::new(
                crate::features::file::repository::FileEmbeddingRepositoryImpl::new(mysql_kind),
            ),
            pool,
            oidc_client,
            jwt_service,
            storage_service,
            temporary_storage_service,
            jti_blacklist_service,
            access_token_duration,
            refresh_token_duration,
            frontend_url,
            embedding_client,
            queue_service,
        }
    }
}
