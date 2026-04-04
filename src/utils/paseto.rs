use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{Duration, Utc};
use pasetors::{Local, claims::{Claims, ClaimsValidationRules}, keys::SymmetricKey, local, token::UntrustedToken, version4::V4};

use crate::{errors::AppError, models::users::TokenClaims};

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

pub fn verify_access_token(token: &str, key_base64: &str) -> Result<TokenClaims, AppError> {
    let key_bytes = BASE64
        .decode(key_base64)
        .map_err(|_| AppError::InvalidToken)?;

    let key = SymmetricKey::<V4>::from(&key_bytes)
        .map_err(|_| AppError::InvalidToken)?;

    let validation_rules = ClaimsValidationRules::new();

    let untrusted = UntrustedToken::<Local, V4>::try_from(token)
        .map_err(|_| AppError::InvalidToken)?;

    let trusted = local::decrypt(&key, &untrusted, &validation_rules, None, None)
        .map_err(|_| AppError::InvalidToken)?;

    let claims = trusted.payload_claims()
        .ok_or(AppError::InvalidToken)?;

    let sub = claims
        .get_claim("sub")
        .and_then(|v| v.as_str())
        .ok_or(AppError::InvalidToken)?
        .to_string();

    let roles = claims
        .get_claim("roles")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        })
        .ok_or(AppError::InvalidToken)?;

    let iat = claims
        .get_claim("iat")
        .and_then(|v| v.as_i64())
        .ok_or(AppError::InvalidToken)?;

    // Parse expiration from ISO 8601 string set by pasetors
    let exp_str = claims
        .get_claim("exp")
        .and_then(|v| v.as_str())
        .ok_or(AppError::InvalidToken)?;

    let exp = chrono::DateTime::parse_from_rfc3339(exp_str)
        .map_err(|_| AppError::InvalidToken)?
        .timestamp();

    if Utc::now().timestamp() > exp {
        return Err(AppError::InvalidToken)
    }

    Ok(TokenClaims{
        sub,
        roles,
        iat,
        exp
    })
}