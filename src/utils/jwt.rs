use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

use crate::errors::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub iat: i64,
    pub exp: i64
}

pub fn create_token(
    user_id: i32,
    role: &str,
    secret: &str,
    expires_in_secs: i64
) -> Result<String, AppError> {
    let now = Utc::now();
    let exp = now + Duration::seconds(expires_in_secs as i64);

    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        iat: now.timestamp(),
        exp: exp.timestamp()
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes())
    ).map_err(|_| AppError::Internal(anyhow::anyhow!("Failed to create token")))
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default()
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::InvalidToken)
}

#[cfg(test)]
mod tests {
    use uuid::timestamp;

    use super::*;

    const TEST_SECRET: &str = "test_secret_key_that-is_long_enough";
    const TEST_USER_ID: i32 = 42;
    const TEST_ROLE: &str = "ADMIN";

    #[test]
    fn test_create_token_returns_non_empty_string(){
        let token = create_token(TEST_USER_ID, TEST_ROLE, TEST_SECRET, 3600).unwrap();

        assert!(!token.is_empty())
    }

    #[test]
    fn test_token_has_three_jwt_segments(){
        let token = create_token(TEST_USER_ID, TEST_ROLE, TEST_SECRET, 3600).unwrap();

        assert_eq!(token.split(".").count(), 3) // JWT format: header.payload.signature
    }

    #[test]
    fn test_verify_valid_token_returns_correct_claims(){
        let token = create_token(TEST_USER_ID, TEST_ROLE, TEST_SECRET, 3600).unwrap();
        let claims = verify_token(&token, TEST_SECRET).unwrap();

        assert_eq!(claims.sub, TEST_USER_ID.to_string());
        assert_eq!(claims.role, TEST_ROLE)
    }

    #[test]
    fn test_verify_token_with_wrong_secret_fails(){
        let token = create_token(TEST_USER_ID, TEST_ROLE, TEST_SECRET, 3600).unwrap();
        let result = verify_token(&token, "wrong_secret");

        assert!(matches!(result, Err(AppError::InvalidToken)))
    }

    #[test]
    fn test_verify_expired_token_fails(){
        let token = create_token(TEST_USER_ID, TEST_ROLE, TEST_SECRET, -100).unwrap();

        let result = verify_token(&token, TEST_SECRET);
        println!("{:?}", result);

        assert!(matches!(result, Err(AppError::InvalidToken)))
    }

    #[test]
    fn test_verify_tampered_token_fails(){
        let token = create_token(TEST_USER_ID, TEST_ROLE, TEST_SECRET, 3600).unwrap();
        // Tamper with the signature (last segment)
        let parts: Vec<&str> = token.split(".").collect();
        let tampered = format!("{}.{}.tampered_signature", parts[0], parts[1]);
        let result = verify_token(&tampered, TEST_SECRET);

        assert!(matches!(result, Err(AppError::InvalidToken)))
    }

    #[test]
    fn test_verify_malformed_token_fails(){
        let result = verify_token("not.a.valid.jwt.token", TEST_SECRET);

        assert!(matches!(result, Err(AppError::InvalidToken)))
    }

    #[test]
    fn test_claims_contain_issued_at(){
        let before = Utc::now().timestamp();
        let token = create_token(TEST_USER_ID, TEST_ROLE, TEST_SECRET, 3600).unwrap();
        let after = Utc::now().timestamp();

        let claims = verify_token(&token, TEST_SECRET).unwrap();

        assert!(claims.iat >= before && claims.iat <= after)
    }

    #[test]
    fn test_different_users_produce_different_tokens(){
        let token1 = create_token(1, TEST_ROLE, TEST_SECRET, 3600).unwrap();
        let token2 = create_token(2, TEST_ROLE, TEST_SECRET, 3600).unwrap();

        assert_ne!(token1, token2);
    }
}