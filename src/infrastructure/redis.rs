use std::time::Duration;

use deadpool_redis::{redis::{AsyncCommands, cmd}, Config as DeadpoolConfig, Pool, Runtime};
use tracing::info;

use crate::{errors::AppError, infrastructure::config::RedisConfig};


pub async fn create_pool(config: &RedisConfig) -> Result<Pool, AppError> {
    if config.url.is_empty() {
        return Err(AppError::Internal(anyhow::anyhow!(
            "REDIS_URLS is empty — at least one Redis URL is required"
        )));
    }

    let url = config.url.clone();
    let cfg = DeadpoolConfig::from_url(url);
    let pool = cfg
        .builder()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis pool builder error: {}", e)))?
        .max_size(config.pool_size)
        .wait_timeout(Some(config.timeout))
        .create_timeout(Some(config.timeout))
        .recycle_timeout(Some(config.timeout))
        .runtime(Runtime::Tokio1)
        .build()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to build Redis pool: {}", e)))?;

    let mut conn = pool
        .get()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis connection failed: {}", e)))?;

    cmd("PING")
        .query_async::<String>(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis PING failed: {}", e)))?;

    info!("Redis connected (pool_size={})", config.pool_size);
    Ok(pool)
}