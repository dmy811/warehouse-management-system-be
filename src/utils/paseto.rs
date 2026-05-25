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
    pub iat: i64,
    pub exp: i64,
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

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
 
    fn test_key() -> String {
        // 32 random bytes encoded as base64
        BASE64.encode([0u8; 32])
    }

    fn test_roles_single() -> Vec<String> {
        vec!["ADMIN".to_string()]
    }

    fn test_roles_multiple() -> Vec<String> {
        vec!["ADMIN".to_string(), "MANAGER".to_string()]
    }
 
    fn test_hmac_secret() -> &'static str {
        "test_hmac_secret_long_enough_32ch"
    }
 
    // ── Access token tests ────────────────────────────────────────────────────
 
    #[test]
    fn test_create_and_verify_single_role() {
        let token = create_access_token(42, &test_roles_single(), &test_key(), 3600).unwrap();
        let claims = verify_access_token(&token, &test_key()).unwrap();

        assert_eq!(claims.sub, "42");
        assert_eq!(claims.roles, vec!["ADMIN"]);
    }


    #[test]
    fn test_create_and_verify_multiple_roles() {
        let token = create_access_token(1, &test_roles_multiple(), &test_key(), 3600).unwrap();
        let claims = verify_access_token(&token, &test_key()).unwrap();

        assert_eq!(claims.roles.len(), 2);
        assert!(claims.roles.contains(&"ADMIN".to_string()));
        assert!(claims.roles.contains(&"MANAGER".to_string()));
    }
 
    #[test]
    fn test_access_token_is_not_jwt() {
        // PASETO tokens start with "v4.local." not "eyJ"
        let token = create_access_token(1, &test_roles_single(), &test_key(), 3600).unwrap();
        assert!(token.starts_with("v4.local."));
        assert!(!token.starts_with("eyJ"));
    }
 
    #[test]
    fn test_expired_access_token_is_rejected() {
        let token = create_access_token(1, &test_roles_single(), &test_key(), -1).unwrap();
        let result = verify_access_token(&token, &test_key());
        assert!(matches!(result, Err(AppError::InvalidToken)));
    }
 
    #[test]
    fn test_wrong_key_rejects_token() {
        let token = create_access_token(1, &test_roles_single(), &test_key(), 3600).unwrap();
        let wrong_key = BASE64.encode([1u8; 32]);
        let result = verify_access_token(&token, &wrong_key);
        assert!(matches!(result, Err(AppError::InvalidToken)));
    }
 
    #[test]
    fn test_tampered_token_is_rejected() {
        let token = create_access_token(1, &test_roles_single(), &test_key(), 3600).unwrap();
        let tampered = format!("{}X", token);
        let result = verify_access_token(&tampered, &test_key());
        assert!(matches!(result, Err(AppError::InvalidToken)));
    }
 
    // ── Refresh token tests ───────────────────────────────────────────────────
 
    #[test]
    fn test_generate_and_verify_refresh_token() {
        let rt = generate_refresh_token(test_hmac_secret(), 32).unwrap();
        let raw = verify_refresh_token(&rt.token, test_hmac_secret()).unwrap();
        assert_eq!(raw, rt.raw_bytes);
    }
 
    #[test]
    fn test_two_refresh_tokens_are_different() {
        let rt1 = generate_refresh_token(test_hmac_secret(), 32).unwrap();
        let rt2 = generate_refresh_token(test_hmac_secret(), 32).unwrap();
        assert_ne!(rt1.token, rt2.token);
    }
 
    #[test]
    fn test_tampered_refresh_token_is_rejected() {
        let rt = generate_refresh_token(test_hmac_secret(), 32).unwrap();
        let tampered = format!("{}X", rt.token);
        let result = verify_refresh_token(&tampered, test_hmac_secret());
        assert!(matches!(result, Err(AppError::InvalidToken)));
    }
 
    #[test]
    fn test_wrong_secret_rejects_refresh_token() {
        let rt = generate_refresh_token(test_hmac_secret(), 32).unwrap();
        let result = verify_refresh_token(&rt.token, "wrong_secret");
        assert!(matches!(result, Err(AppError::InvalidToken)));
    }
 
    #[test]
    fn test_hash_refresh_token_is_deterministic() {
        let raw = vec![1u8, 2, 3, 4];
        let hash1 = hash_refresh_token(&raw);
        let hash2 = hash_refresh_token(&raw);
        assert_eq!(hash1, hash2);
    }
 
    #[test]
    fn test_hash_refresh_token_different_inputs_differ() {
        let hash1 = hash_refresh_token(&[1u8; 32]);
        let hash2 = hash_refresh_token(&[2u8; 32]);
        assert_ne!(hash1, hash2);
    }
}                                                                                                                                           