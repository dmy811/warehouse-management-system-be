// use std::collections::HashSet;

// use anyhow::{Context};
// use chrono::{Duration, Utc};
// use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
// use serde::{Deserialize, Serialize};

// #[derive(Clone)]
// pub struct JwtConfig {
//     pub secret: String,
//     pub expiration_in_minutes: i64,
//     pub issuer: String
// } 


// impl JwtConfig {
//     pub fn from_env() -> anyhow::Result<Self> {
//         dotenvy::dotenv().ok();
//         let secret: String = std::env::var("JWT_SECRET").context("JWT_SECRET env not yet set")?;
//         let expiration_in_minutes: i64 = std::env::var("JWT_EXPIRES_MINUTES")
//             .ok()
//             .and_then(|s| s.parse::<i64>().ok())
//             .unwrap_or(60 * 24);
//         let issuer = std::env::var("JWT_ISSUER").unwrap_or_else(|_| "mini-warehouse-wms".to_string());
//         Ok(Self { secret, expiration_in_minutes, issuer })
//     }

//     pub fn encoding_key(&self) -> EncodingKey {
//         EncodingKey::from_secret(self.secret.as_bytes())
//     }

//     pub fn decoding_key(&self) -> DecodingKey {
//         DecodingKey::from_secret(self.secret.as_bytes())
//     }
// }

// #[derive(Serialize, Deserialize, Clone)]
// pub struct Claims {
//     pub sub: String,
//     pub exp: i64,
//     pub iat: i64,
//     pub iss: String,
//     pub name: String,
//     pub role: String
// }

// pub fn sign_jwt(cfg: &JwtConfig, user_id: i64, name: &str, role: &str) -> anyhow::Result<String> {
//     let now = Utc::now();
//     let exp = now + Duration::minutes(cfg.expiration_in_minutes);
//     let claims = Claims {
//         sub: user_id.to_string(),
//         name: name.to_string(),
//         role: role.to_string(),
//         iss: cfg.issuer.clone(),
//         iat: now.timestamp(),
//         exp: exp.timestamp(),
//     };
//     let mut header = Header::default();
//     header.alg = jsonwebtoken::Algorithm::HS256;
    
//     let token = jsonwebtoken::encode(&header, &claims, &cfg.encoding_key())?;
//     Ok(token)
// }

// pub fn verify_jwt(cfg: &JwtConfig, token: &str) -> anyhow::Result<Claims> {
//     let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
//     validation.set_required_spec_claims(&["exp", "iat",  "iss", "sub"]);
//     validation.validate_exp = true;
//     validation.iss = Some(HashSet::from([cfg.issuer.clone()]));

//     let data = jsonwebtoken::decode::<Claims>(token, &cfg.decoding_key(), &validation)?;
//     Ok(data.claims)
// }