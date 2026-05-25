// use chrono::{Duration, Utc};
// use serde::{Deserialize, Serialize};
// use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

// use crate::errors::AppError;

// #[derive(Debug, Serialize, Deserialize)]
// pub struct Claims {
//     pub sub: String,
//     pub roles: Vec<String>,
//     pub iat: i64,
//     pub exp: i64
// }

// pub fn create_access_token(
//     user_id: i64,
//     roles: &[String],
//     secret: &str,
//     expires_in_secs: i64
// ) -> Result<String, AppError> {
//     let now = Utc::now();
//     let exp = now + Duration::seconds(expires_in_secs as i64);

//     let claims = Claims {
//         sub: user_id.to_string(),
//         roles: roles.to_vec(),
//         iat: now.timestamp(),
//         exp: exp.timestamp()
//     };

//     encode(
//         &Header::default(),
//         &claims,
//         &EncodingKey::from_secret(secret.as_bytes())
//     ).map_err(|_| AppError::Internal(anyhow::anyhow!("Failed to create token")))
// }

// pub fn verify_access_token(token: &str, secret: &str) -> Result<Claims, AppError> {
//     decode::<Claims>(
//         token,
//         &DecodingKey::from_secret(secret.as_bytes()),
//         &Validation::default()
//     )
//     .map(|data| data.claims)
//     .map_err(|_| AppError::InvalidToken)
// }
