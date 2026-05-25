use deadpool_redis::{redis::{AsyncCommands, cmd}, Config as DeadpoolConfig, Pool, Runtime};
use tracing::info;

use crate::{errors::AppError, infrastructure::config::RedisConfig};


pub async fn create_pool(redis_config: &RedisConfig) -> Result<Pool, AppError> {
    if redis_config.url.is_empty() {
        return Err(AppError::Internal(anyhow::anyhow!(
            "REDIS_URLS is empty — at least one Redis URL is required"
        )));
    }

    let url = redis_config.url.clone();
    let cfg = DeadpoolConfig::from_url(url);
    let pool = cfg
        .builder()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis pool builder error: {}", e)))?
        .max_size(redis_config.pool_size)
        .wait_timeout(Some(redis_config.timeout))
        .create_timeout(Some(redis_config.timeout))
        .recycle_timeout(Some(redis_config.timeout))
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

    info!("Redis connected (pool_size={})", redis_config.pool_size);
    Ok(pool)
}

pub async fn set_ex(
    pool: &Pool,
    key: &str,
    value: &str,
    ttl_secs: u64
) -> Result<(), AppError> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis get conn: {}", e)))?;

    conn.set_ex::<_, _, ()>(key, value, ttl_secs)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis SET EX failed: {}", e)))
}

pub async fn get(pool: &Pool, key: &str) -> Result<Option<String>, AppError> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis get conn: {}", e)))?;
 
    conn.get::<_, Option<String>>(key)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis GET failed: {}", e)))
}

pub async fn del(pool: &Pool, key: &str) -> Result<bool, AppError> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis get conn: {}", e)))?;
 
    let deleted: i64 = conn
        .del(key)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis DEL failed: {}", e)))?;
 
    Ok(deleted > 0)
}

/// Increment a counter. Returns the new value.
pub async fn incr(pool: &Pool, key: &str) -> Result<i64, AppError> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis get conn: {}", e)))?;
 
    conn.incr::<_, _, i64>(key, 1)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis INCR failed: {}", e)))
}
 
/// Set TTL on an existing key. Used to set expiry after INCR.
pub async fn expire(pool: &Pool, key: &str, ttl_secs: u64) -> Result<(), AppError> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis get conn: {}", e)))?;
 
    conn.expire::<_, ()>(key, ttl_secs as i64)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis EXPIRE failed: {}", e)))
}

pub async fn exists(pool: &Pool, key: &str) -> Result<bool, AppError> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis get conn: {}", e)))?;
 
    let result: bool = conn
        .exists(key)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis EXISTS failed: {}", e)))?;
 
    Ok(result)
}
 
 pub mod keys {
    /// Refresh token storage: token_hash → user_id
    pub fn refresh_token(token_hash: &str) -> String {
        format!("wms:refresh_token:{}", token_hash)
    }
 
    /// Failed login counter per email
    pub fn failed_login(email: &str) -> String {
        format!("wms:failed_login:{}", email)
    }
 
    /// Account lockout flag per email
    pub fn lockout(email: &str) -> String {
        format!("wms:lockout:{}", email)
    }
 
    /// Rate limit counter per IP
    pub fn rate_limit(ip: &str) -> String {
        format!("wms:rate_limit:{}", ip)
    }
 
    /// Blacklisted access tokens (for logout)
    pub fn token_blacklist(jti: &str) -> String {
        format!("wms:blacklist:{}", jti)
    }
}