pub async fn csrf(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::response::IntoResponse as _;

    if matches!(
        req.method(),
        &axum::http::Method::GET | &axum::http::Method::HEAD | &axum::http::Method::OPTIONS
    ) {
        return next.run(req).await;
    }

    let jar = axum_extra::extract::CookieJar::from_headers(req.headers());
    let cookie_csrf = jar.get("csrf").map(|c| c.value().to_string());

    let header_csrf = req
        .headers()
        .get("x-csrf-token")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string());

    match (cookie_csrf, header_csrf) {
        (Some(cookie), Some(header)) if cookie == header => next.run(req).await,
        _ => axum::http::StatusCode::FORBIDDEN.into_response(),
    }
}
