pub fn public_router<S: Clone + Send + Sync + 'static>() -> utoipa_axum::router::OpenApiRouter<S> {
    utoipa_axum::router::OpenApiRouter::new().routes(utoipa_axum::routes!(health))
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct HealthResponse {
    pub status: String,
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = inline(HealthResponse)),
    )
)]
pub async fn health() -> axum::Json<HealthResponse> {
    axum::Json(HealthResponse {
        status: "ok".to_string(),
    })
}
