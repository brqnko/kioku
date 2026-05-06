pub mod middleware;
pub mod schema;

pub type HandlerResult<T> = schema::HandlerResult<T>;

mod docs {
    use crate::features::chatbot::handler::*;
    use crate::features::file::handler::*;
    use crate::features::misc::handler;
    use crate::features::podcast::handler::*;
    use crate::features::project::handler::*;
    use crate::features::user::handler::*;
    use crate::server::schema::*;

    #[derive(utoipa::OpenApi)]
    #[openapi(
        paths(
            handler::health,
            oidc_start,
            oidc_callback,
            logout,
            refresh,
            get_user_profile,
            update_user_profile,
            remove_user,
            list_sessions,
            revoke_session,
            revoke_all_sessions,
            get_dashboard,
            create_project,
            list_projects,
            get_project,
            update_project,
            remove_project,
            request_upload_url,
            create_file,
            get_file_content,
            update_file,
            update_file_text,
            remove_file,
            create_folder,
            get_folder,
            update_folder,
            remove_folder,
            get_folder_ancestors,
            get_file_ancestors,
            list_project_children,
            list_folder_children,
            create_podcast,
            list_podcasts,
            get_podcast,
            update_podcast,
            remove_podcast,
            create_chat,
            list_chats,
            get_chat,
            send_message,
            remove_chat,
        ),
        components(schemas(ErrorBody,))
    )]
    pub struct ApiDoc;
}

pub fn api_doc() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi as _;

    docs::ApiDoc::openapi()
}

pub fn router(app: std::sync::Arc<crate::app::App>) -> axum::Router {
    use utoipa::OpenApi as _;

    let protected = crate::features::user::handler::protected_router()
        .merge(crate::features::project::handler::protected_router())
        .merge(crate::features::file::handler::protected_router())
        .merge(crate::features::podcast::handler::protected_router())
        .merge(crate::features::chatbot::handler::protected_router())
        .layer(axum::middleware::from_fn(middleware::csrf::csrf))
        .layer(axum::middleware::from_fn_with_state(
            app.clone(),
            middleware::auth::auth,
        ));

    let (router, api) =
        utoipa_axum::router::OpenApiRouter::<std::sync::Arc<crate::app::App>>::with_openapi(
            docs::ApiDoc::openapi(),
        )
        .merge(crate::features::misc::handler::public_router())
        .merge(crate::features::user::handler::public_router())
        .merge(protected)
        .split_for_parts();

    let router = router
        .merge(<utoipa_redoc::Redoc<_> as utoipa_redoc::Servable<_>>::with_url("/redoc", api))
        .with_state(app)
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(
                    tower_http::trace::DefaultMakeSpan::new().level(tracing::Level::INFO),
                )
                .on_request(tower_http::trace::DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(
                    tower_http::trace::DefaultOnResponse::new().level(tracing::Level::INFO),
                ),
        )
        .layer(tower_http::request_id::PropagateRequestIdLayer::x_request_id())
        .layer(tower_http::request_id::SetRequestIdLayer::x_request_id(
            tower_http::request_id::MakeRequestUuid,
        ));

    #[cfg(debug_assertions)]
    let router = router.layer(axum::middleware::from_fn(middleware::dev_delay::dev_delay));

    router
}
