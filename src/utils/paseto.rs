use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{Duration, Utc};
use pasetors::{claims::Claims, keys::SymmetricKey, local, version4::V4};

use crate::errors::AppError;

pub fn create_access_token(
    user_id: i64,
    roles: &[String],
    key_base64: &str,
    expires_in_secs: i64
) -> Result<String, AppError> {
    let key_bytes = BASE64
        .decode(key_base64)
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid PASETO key: base 64 decode failed")))?;

    let key = SymmetricKey::<V4>::from(&key_bytes)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid PASETO key: {}", e)))?;

    let now = Utc::now();
    let exp = now + Duration::seconds(expires_in_secs);

    let mut claims = Claims::new()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create claims: {}", e)))?;

    claims.subject(&user_id.to_string())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to set sub: {}", e)))?;
    claims.add_additional("roles", roles.clone())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to set roles: {}", e)))?;
    claims.add_additional("iat", now.timestamp())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to set iat: {}", e)))?;
    claims.expiration(&exp.to_rfc3339())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to set exp: {}", e)))?;

    local::encrypt(&key, &claims, None, None)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to encrypt token: {}", e)))
}