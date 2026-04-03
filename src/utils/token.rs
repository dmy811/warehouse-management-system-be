use pasetors::keys::SymmetricKey;
use pasetors::version4::V4;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::infrastructure::config::Config;
use crate::models::User;

pub struct AccessTokenService {
    symmetric_key: SymmetricKey<V4>,
    ttl_seconds: i64
}

impl AccessTokenService {
    pub fn new(config: &Config) -> AppResult<Self> {
        let key_bytes = config.paseto_symmetric_key.as_slice();
        Ok(Self {
            symmetric_key: SymmetricKey::<V4>::from(key_bytes)
                .map_err(|e| AppError::Internal(anyhow::anyhow!(format!("Invalid PASETO key length: {}", e))))?,
            ttl_seconds: config.access_token_ttl_seconds,
        })

    }

    pub fn generate_access_token(&self, user: &User) -> AppResult<(String, String)> {
        let token_id = Uuid::now_v7().to_string();
        let now = OffsetDateTime::now_utc();
    }
}