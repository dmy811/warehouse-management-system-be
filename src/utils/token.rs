// use pasetors::keys::SymmetricKey;
// use pasetors::local::encrypt;
// use pasetors::version4::V4;
// use time::OffsetDateTime;
// use uuid::Uuid;

// use crate::errors::{AppError, AppResult};
// use crate::infrastructure::config::Config;
// use crate::models::{UserWithRole};
// use crate::models::users::AccessTokenClaims;

// # Generate 32-byte key untuk PASETO v4.local
// openssl rand -hex 32

// # Generate 64-byte key untuk HMAC (refresh token)
// openssl rand -hex 64

// generate_keys.rs
// Jalankan: cargo run --bin generate_keys

// use rand::rngs::OsRng;
// use rand::RngCore;

// fn main() {
//     println!("========================================");
//     println!("KEY GENERATION FOR PRODUCTION");
//     println!("========================================\n");
    
//     // PASETO v4.local key (32 bytes)
//     let mut paseto_key = [0u8; 32];
//     OsRng.fill_bytes(&mut paseto_key);
//     let paseto_hex = hex::encode(paseto_key);
    
//     println!("PASETO_SYMMETRIC_KEY=\"{}\"", paseto_hex);
//     println!("# Panjang: {} karakter hex (32 bytes)", paseto_hex.len());
//     println!();
    
//     // HMAC key (64 bytes recommended)
//     let mut hmac_key = [0u8; 64];
//     OsRng.fill_bytes(&mut hmac_key);
//     let hmac_hex = hex::encode(hmac_key);
    
//     println!("REFRESH_TOKEN_HMAC_SECRET=\"{}\"", hmac_hex);
//     println!("# Panjang: {} karakter hex (64 bytes)", hmac_hex.len());
//     println!();
    
//     // Optional: Also generate base64 versions
//     let paseto_b64 = base64::encode(&paseto_key);
//     let hmac_b64 = base64::encode(&hmac_key);
    
//     println!("--- Base64 versions (for some config formats) ---");
//     println!("PASETO_SYMMETRIC_KEY_BASE64=\"{}\"", paseto_b64);
//     println!("REFRESH_TOKEN_HMAC_SECRET_BASE64=\"{}\"", hmac_b64);
// }

// [[bin]]
// name = "generate_keys"
// path = "generate_keys.rs"

// [dependencies]
// rand = { version = "0.8", features = ["getrandom"] }
// hex = "0.4"
// base64 = "0.22"

// cargo run --bin generate_keys

// pub struct AccessTokenService {
//     symmetric_key: SymmetricKey<V4>,
//     ttl_seconds: i64
// }

// impl AccessTokenService {
//     pub fn new(config: &Config) -> AppResult<Self> {
//         let key_bytes = &config.paseto_symmetric_key;
//         if key_bytes.len() != 32 {
//             return Err(AppError::Internal(anyhow::anyhow!(format!(
//                 "PASETO v4 key must be 32 bytes, got {}",
//                 key_bytes.len()
//             ))));
//         }

//         let symmetric_key = SymmetricKey::<V4>::from(key_bytes.as_slice())
//             .map_err(|e| AppError::Internal(anyhow::anyhow!(format!("Generating paseto symetric key error: {}", e))))?;

//         Ok(Self {
//             symmetric_key,
//             ttl_seconds: config.access_token_ttl_seconds,
//         })

//     }

//     pub fn generate_access_token(&self, user: &UserWithRole) -> AppResult<(String, String)> {
//         let token_id = Uuid::now_v7().to_string();
//         let now = OffsetDateTime::now_utc();
//         let exp = now + time::Duration::seconds(self.ttl_seconds);

//         let claims = AccessTokenClaims {
//             sub: user.id.to_string(),
//             jti: token_id.to_string(),
//             exp: exp.unix_timestamp(),
//             iat: now.unix_timestamp(),
//             roles: user.roles.clone().unwrap_or_default(),
//             version: 4
//         };

//         let payload = serde_json::to_vec(&claims)
//             .map_err(|e| AppError::Internal(anyhow::anyhow!(format!("Serialization failed on generating access token: {}", e))))?;

//         let token = encrypt(&self.symmetric_key, &claims, None, None)
//             .map_err(|e| AppError::Internal(anyhow::anyhow!(format!("Paseto encryption failed on generating access token: {}", e)))?;

//         Ok((token, token_id))
//     }
// }