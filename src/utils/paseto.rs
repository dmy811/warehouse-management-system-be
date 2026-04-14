use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use pasetors::{Local, claims::{Claims, ClaimsValidationRules}, keys::SymmetricKey, local, token::UntrustedToken, version4::V4};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::{errors::AppError};

// --- Acess Token Paseto V4 local -------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub roles: Vec<String>,
    pub exp: i64,
    pub iat: i64,
}

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
    claims.add_additional("roles", roles.to_vec())
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

// --- Refresh Token HMAC signed opaque token -------------------------------------------
#[derive(Debug, Clone)]
pub struct RefreshToken {
    pub token: String,
    pub raw_bytes: Vec<u8>
}

pub fn generate_refresh_token(
    hmac_secret: &str,
    length_bytes: usize
) -> Result<RefreshToken, AppError> {
    let mut raw_bytes = vec![0u8; length_bytes];
    rand::thread_rng().fill_bytes(&mut raw_bytes);

    let signature = hmac_sign(&raw_bytes, hmac_secret)?;

    let token = format!(
        "{}.{}", BASE64.encode(&raw_bytes), BASE64.encode(&signature)
    );

    Ok(RefreshToken { token, raw_bytes })
}

pub fn verify_refresh_token(
    token: &str,
    hmac_secret: &str
) -> Result<Vec<u8>, AppError> {
    let parts: Vec<&str> = token.splitn(2, ".").collect();
    if parts.len() != 2 {
        return Err(AppError::InvalidToken)
    }

    let raw_bytes = BASE64.decode(parts[0]).map_err(|_| AppError::InvalidToken)?;
    let provided_sig = BASE64.decode(parts[1]).map_err(|_| AppError::InvalidToken)?;

    let expected_sig = hmac_sign(&raw_bytes, hmac_secret)?;

    if !constant_time_eq(&provided_sig, &expected_sig) {
        return Err(AppError::InvalidToken)
    }

    Ok(raw_bytes)
}

pub fn hash_refresh_token(raw_bytes: &[u8]) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(raw_bytes);

    hex::encode(hasher.finalize())
}

fn hmac_sign(data: &[u8], secret: &str) -> Result<Vec<u8>, AppError> {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid HMAC key length")))?;
    mac.update(data);
    Ok(mac.finalize().into_bytes().to_vec())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

