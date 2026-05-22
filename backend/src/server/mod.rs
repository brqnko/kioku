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

    pub struct SecurityAddon;

    impl utoipa::Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};

            let components = openapi
                .components
                .get_or_insert_with(utoipa::openapi::Components::new);
            components.add_security_scheme(
                "cookieAuth",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("access_token"))),
            );
            components.add_security_scheme(
                "csrfToken",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("x-csrf-token"))),
            );
        }
    }

    #[derive(utoipa::OpenApi)]
    #[openapi(
        modifiers(&SecurityAddon),
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
            get_rate_limits,
            create_project,
            list_projects,
            get_project,
            update_project,
            remove_project,
            request_upload_url,
            create_file,
            get_file_content,
            get_file_raw,
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
            run_code,
            list_compilers,
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

struct HeaderExtractor<'a>(&'a axum::http::HeaderMap);

impl<'a> opentelemetry::propagation::Extractor for HeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|k| k.as_str()).collect()
    }
}

fn make_otel_span<B>(req: &axum::http::Request<B>) -> tracing::Span {
    use tracing_opentelemetry::OpenTelemetrySpanExt as _;

    let span = tracing::info_span!(
        "http_request",
        otel.kind = "server",
        http.request.method = %req.method(),
        url.path = %req.uri().path(),
    );
    let parent = opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(req.headers()))
    });
    let _ = span.set_parent(parent);
    span
}

fn http_duration_histogram() -> &'static opentelemetry::metrics::Histogram<f64> {
    use std::sync::OnceLock;
    static H: OnceLock<opentelemetry::metrics::Histogram<f64>> = OnceLock::new();
    H.get_or_init(|| {
        opentelemetry::global::meter("kioku-backend")
            .f64_histogram("http.server.request.duration")
            .with_unit("s")
            .build()
    })
}

async fn metrics_mw(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let start = std::time::Instant::now();
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let response = next.run(req).await;
    let status = response.status().as_u16() as i64;
    http_duration_histogram().record(
        start.elapsed().as_secs_f64(),
        &[
            opentelemetry::KeyValue::new("http.request.method", method.to_string()),
            opentelemetry::KeyValue::new("url.path", path),
            opentelemetry::KeyValue::new("http.response.status_code", status),
        ],
    );
    response
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

    router
        .merge(<utoipa_redoc::Redoc<_> as utoipa_redoc::Servable<_>>::with_url("/redoc", api))
        .with_state(app)
        .layer(axum::extract::DefaultBodyLimit::max(32 * 1024 * 1024))
        .layer(axum::middleware::from_fn(metrics_mw))
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(make_otel_span)
                .on_response(
                    tower_http::trace::DefaultOnResponse::new().level(tracing::Level::INFO),
                ),
        )
        .layer(tower_http::request_id::PropagateRequestIdLayer::x_request_id())
        .layer(tower_http::request_id::SetRequestIdLayer::x_request_id(
            tower_http::request_id::MakeRequestUuid,
        ))
}
