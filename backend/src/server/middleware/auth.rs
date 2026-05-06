pub async fn auth(
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::response::IntoResponse as _;

    let jar = axum_extra::extract::CookieJar::from_headers(req.headers());
    let token = match jar.get("access_token").map(|c| c.value().to_string()) {
        Some(t) => t,
        None => return axum::http::StatusCode::UNAUTHORIZED.into_response(),
    };

    let input = crate::features::user::usecase::VerifyAccessTokenInput {
        access_token: token,
    };

    match crate::features::user::usecase::verify_access_token(&app, input).await {
        Ok(Ok(output)) => {
            req.extensions_mut().insert(output.user_id);
            next.run(req).await
        }
        Ok(Err(_)) => axum::http::StatusCode::UNAUTHORIZED.into_response(),
        Err(err) => {
            tracing::error!("{:?}", err);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
