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
    pub podcast_repository: std::sync::Arc<
        dyn crate::features::podcast::repository::PodcastRepository<sqlx::MySqlConnection>,
    >,
    pub podcast_query_service:
        std::sync::Arc<dyn crate::features::podcast::query_service::QueryService>,
    pub chat_repository: std::sync::Arc<
        dyn crate::features::chatbot::repository::ChatRepository<sqlx::MySqlConnection>,
    >,
    pub chat_query_service:
        std::sync::Arc<dyn crate::features::chatbot::query_service::QueryService>,
    pub embedding_client: std::sync::Arc<dyn crate::util::embedding::EmbeddingClient>,
    pub llm_client: std::sync::Arc<dyn crate::util::llm::LLMClient>,
    pub md_convert_service: std::sync::Arc<dyn crate::util::mdutil::MdConvertService>,
    pub podcast_request_service:
        std::sync::Arc<dyn crate::util::podcast_request::PodcastRequestService>,
    pub code_runner_client: std::sync::Arc<dyn crate::util::code_runner::CodeRunnerClient>,
    pub locker: std::sync::Arc<dyn crate::util::ad_lock::Locker>,
    pub sgi_url: String,
    pub sgi_token: String,
}

pub struct AppArgs {
    pub pool: sqlx::Pool<sqlx::MySql>,
    pub oidc_client: std::sync::Arc<dyn crate::features::user::service::OIDCClient>,
    pub jwt_service: std::sync::Arc<dyn crate::util::jwt::JWTService>,
    pub storage_service: std::sync::Arc<dyn crate::util::storage::StorageService>,
    pub temporary_storage_service: std::sync::Arc<dyn crate::util::storage::StorageService>,
    pub jti_blacklist_service: std::sync::Arc<dyn crate::util::jti_blacklist::JtiBlacklistService>,
    pub embedding_client: std::sync::Arc<dyn crate::util::embedding::EmbeddingClient>,
    pub llm_client: std::sync::Arc<dyn crate::util::llm::LLMClient>,
    pub md_convert_service: std::sync::Arc<dyn crate::util::mdutil::MdConvertService>,
    pub podcast_request_service:
        std::sync::Arc<dyn crate::util::podcast_request::PodcastRequestService>,
    pub code_runner_client: std::sync::Arc<dyn crate::util::code_runner::CodeRunnerClient>,
    pub mysql_kind: crate::util::dialect::MySQLKind,
    pub access_token_duration: chrono::Duration,
    pub refresh_token_duration: chrono::Duration,
    pub frontend_url: String,
    pub sgi_url: String,
    pub sgi_token: String,
}

impl App {
    pub fn new(args: AppArgs) -> Self {
        let AppArgs {
            pool,
            oidc_client,
            jwt_service,
            storage_service,
            temporary_storage_service,
            jti_blacklist_service,
            embedding_client,
            llm_client,
            md_convert_service,
            podcast_request_service,
            code_runner_client,
            mysql_kind,
            access_token_duration,
            refresh_token_duration,
            frontend_url,
            sgi_url,
            sgi_token,
        } = args;
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
            podcast_repository: Arc::new(
                crate::features::podcast::repository::PodcastRepositoryImpl::new(),
            ),
            podcast_query_service: Arc::new(
                crate::features::podcast::query_service::QueryServiceImpl::new(pool.clone()),
            ),
            chat_repository: Arc::new(
                crate::features::chatbot::repository::ChatRepositoryImpl::new(),
            ),
            chat_query_service: Arc::new(
                crate::features::chatbot::query_service::QueryServiceImpl::new(pool.clone()),
            ),
            locker: Arc::new(crate::util::ad_lock::MySqlLocker::new()),
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
            llm_client,
            md_convert_service,
            podcast_request_service,
            code_runner_client,
            sgi_url,
            sgi_token,
        }
    }
}
