use anyhow::Result;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tracing::info;

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    info!("Connecting to database...");

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .idle_timeout(std::time::Duration::from_secs(300))
        .connect(database_url)
        .await?;

    info!("Database connected successfully");
    Ok(pool)
}