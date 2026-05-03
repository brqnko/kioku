use sqlx::Connection as _;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")?;
    let pool = sqlx::MySqlPool::connect(&database_url).await?;

    pool.acquire().await?.ping().await?;
    tracing::info!("ping ok");

    let row = sqlx::query!("SELECT 1 AS one").fetch_one(&pool).await?;
    tracing::info!("SELECT 1 = {:?}", row.one);

    Ok(())
}
