pub async fn dev_delay(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use rand::RngExt as _;
    let ms: u64 = rand::rng().random_range(300..=500);
    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
    next.run(req).await
}
