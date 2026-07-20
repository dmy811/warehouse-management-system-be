use std::sync::Arc;

use async_trait::async_trait;
use deadpool_redis::Pool as RedisPool;
use tracing::{info, warn};

use crate::{dtos::{AuthResponse, LoginRequest, UserResponse, auth_dto::UpdatePasswordRequest, user_dto::UpdateUserRequest}, errors::{AppError, AppResult}, infrastructure::{config::Config, redis::{self, keys}}, repositories::{user_repository::UserRepositoryTrait}, utils::{crypto::verify_password, paseto::{create_access_token, generate_refresh_token, hash_refresh_token, verify_refresh_token}}};

const ACCESS_TOKEN_TTL_SECS: i64 = 15 * 60; // 15 minutes
// Dummy hash konstan untuk mencegah Timing Attack jika email tidak terdaftar
const DUMMY_HASH: &str = "$2b$12$6UxvE8rXmFp.FhWkY5XvGu9ZzG/B5R5iS3B0qQ7xR8W/yE1mG1b2C"; 

#[async_trait]
pub trait AuthServiceTrait: Send + Sync {
    async fn login(&self, req: LoginRequest) -> AppResult<(AuthResponse, String)>;
    async fn get_profile(&self, user_id: i64) -> AppResult<UserResponse>;
    async fn update_profile(&self, id: i64, req: UpdateUserRequest) -> AppResult<UserResponse>;
    async fn update_profile_photo(&self, user_id: i64, photo_url: &str) -> AppResult<()>;
    async fn delete_profile_photo(&self, user_id: i64) -> AppResult<()>;
    async fn update_profile_password(&self, user_id: i64, req: UpdatePasswordRequest) -> AppResult<()>;
    async fn refresh(&self, refresh_token_cookie: &str) -> AppResult<(AuthResponse, String)>;
    async fn logout(&self, refresh_token_cookie: &str) -> AppResult<()>;
}

pub struct AuthService<U: UserRepositoryTrait> {
    user_repo: Arc<U>,
    config: Arc<Config>,
    redis: Arc<RedisPool>
}

impl<U: UserRepositoryTrait> AuthService<U> {
    pub fn new(user_repo: Arc<U>, config: Arc<Config>, redis: Arc<RedisPool>) -> Self {
        Self { user_repo, config, redis }
    }

    async fn record_failed_attempt(&self, email: &str) -> AppResult<()> {
        let failed_key = keys::failed_login(email);
        let count = redis::incr(&self.redis, &failed_key).await?;
        
        // Selalu perbarui atau set expire untuk mengantisipasi race condition
        let ttl = self.config.auth.lockout_seconds() * 2;
        redis::expire(&self.redis, &failed_key, ttl).await?;

        if count >= self.config.auth.failed_login_threshold as i64 {
            let lockout_key = keys::lockout(email);
            redis::set_ex(&self.redis, &lockout_key, "1", self.config.auth.lockout_seconds()).await?;

            warn!(
                email = %email,
                count = count,
                lockout_minutes = self.config.auth.failed_login_lockout_minutes,
                "Account locked due to repeated failed login attempts"
            );
        }

        Ok(())
    }
}

#[async_trait]
impl<U: UserRepositoryTrait> AuthServiceTrait for AuthService<U> {
    async fn login(&self, req: LoginRequest) -> AppResult<(AuthResponse, String)> {
        let lockout_key = keys::lockout(&req.email);
        if redis::exists(&self.redis, &lockout_key).await? {
            warn!(email = %req.email, "Login attempt on locked account");
            return Err(AppError::TooManyRequests("Account temporarily locked. Try again later.".to_string()));
        }

        let user_opt = self.user_repo.find_user_by_email(&req.email).await?;
        let generic_error = || AppError::InvalidCredentials("Invalid email or password".to_string());

        // FIX TIMING ATTACK: Ambil hash asli atau gunakan dummy hash jika email tidak ada
        let password_hash = match &user_opt {
            Some(u) => u.password.clone(),
            None => DUMMY_HASH.to_string(),
        };

        let password = req.password.clone();
        let is_valid = tokio::task::spawn_blocking(move || verify_password(&password, &password_hash))
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Thread join error: {}", e)))??;

        // Jika password salah ATAU user-nya memang dari awal tidak ada
        if !is_valid || user_opt.is_none() {
            self.record_failed_attempt(&req.email).await?;
            warn!(email = %req.email, "Failed login attempt");
            return Err(generic_error());
        }

        let user = user_opt.unwrap(); // Aman karena sudah divalidasi di atas
        let failed_key = keys::failed_login(&req.email);
        redis::del(&self.redis, &failed_key).await?;

        info!(user_id = user.id, "User logged in");
        let roles = user.roles.clone().unwrap_or_default();
        let access_token = create_access_token(user.id, &roles, &self.config.auth.paseto_key, ACCESS_TOKEN_TTL_SECS)?;
        let refresh_token = generate_refresh_token(&self.config.auth.refresh_token_hmac_secret, self.config.auth.refresh_token_length_bytes)?;

        let refresh_token_hash = hash_refresh_token(&refresh_token.raw_bytes);
        let redis_key = keys::refresh_token(&refresh_token_hash);

        redis::set_ex(&self.redis, &redis_key, &user.id.to_string(), self.config.auth.refresh_token_ttl_seconds()).await?;

        Ok((AuthResponse::new(access_token, user), refresh_token.token))
    }

    async fn logout(&self, refresh_token_cookie: &str) -> AppResult<()> {
        let raw_bytes = verify_refresh_token(refresh_token_cookie, &self.config.auth.refresh_token_hmac_secret)
            .unwrap_or_default();

        if !raw_bytes.is_empty() {
            let refresh_token_hash = hash_refresh_token(&raw_bytes);
            redis::del(&self.redis, &keys::refresh_token(&refresh_token_hash)).await?;
        }
        Ok(())
    }

    async fn refresh(&self, refresh_token_cookie: &str) -> AppResult<(AuthResponse, String)> {
        let raw_bytes = verify_refresh_token(refresh_token_cookie, &self.config.auth.refresh_token_hmac_secret)?;

        let refresh_token_hash = hash_refresh_token(&raw_bytes);
        let redis_key = keys::refresh_token(&refresh_token_hash);

        let user_id_str = redis::get(&self.redis, &redis_key).await?.ok_or(AppError::InvalidToken)?;
        let user_id: i64 = user_id_str.parse().map_err(|_| AppError::InvalidToken)?;

        redis::del(&self.redis, &redis_key).await?;

        let user = self.user_repo.find_user_by_id(user_id).await?.ok_or_else(|| AppError::NotFound("User".to_string()))?;
        let roles = user.roles.clone().unwrap_or_default();

        let access_token = create_access_token(user.id, &roles, &self.config.auth.paseto_key, ACCESS_TOKEN_TTL_SECS)?;
        let refresh_token = generate_refresh_token(&self.config.auth.refresh_token_hmac_secret, self.config.auth.refresh_token_length_bytes)?;

        let new_token_hash = hash_refresh_token(&refresh_token.raw_bytes);
        let new_redis_key = keys::refresh_token(&new_token_hash);
        redis::set_ex(&self.redis, &new_redis_key, &user_id.to_string(), self.config.auth.refresh_token_ttl_seconds()).await?;

        info!(user_id = user_id, "Token refreshed");
        Ok((AuthResponse::new(access_token, user), refresh_token.token))
    }

    async fn get_profile(&self, user_id: i64) -> AppResult<UserResponse> {
        let user = self.user_repo.find_user_by_id(user_id).await?.ok_or_else(|| AppError::NotFound("User".to_string()))?;
        Ok(UserResponse::from(user))
    }

    async fn update_profile(&self, id: i64, req: UpdateUserRequest) -> AppResult<UserResponse> {
        self.user_repo.find_user_by_id(id).await?.ok_or_else(|| AppError::NotFound(format!("User with id {}", id)))?;

        if let Some(email) = &req.email {
            if self.user_repo.check_email_exists(email, Some(id)).await? {
                return Err(AppError::Conflict(format!("Email '{}' is already registered", email)));
            }
        }
        
        let user = self.user_repo
            .update_user(id, req.name.as_deref(), req.email.as_deref(), req.phone.as_deref())
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User with id {}", id)))?;
        
        Ok(UserResponse::from(user))
    }

    async fn update_profile_photo(&self, user_id: i64, photo_url: &str) -> AppResult<()> {
        self.user_repo.find_user_by_id(user_id).await?.ok_or_else(|| AppError::NotFound("User".to_string()))?;
        self.user_repo.update_user_photo(user_id, photo_url).await
    }

    async fn delete_profile_photo(&self, user_id: i64) -> AppResult<()> {
        self.user_repo
            .find_user_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User".to_string()))?;
 
        self.user_repo.clear_user_photo(user_id).await
    }

    async fn update_profile_password(&self, user_id: i64, req: UpdatePasswordRequest) -> AppResult<()> {
        let user = self.user_repo
            .find_user_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User".to_string()))?;

        let old_password = req.old_password.clone();
        let user_password_hash = user.password.clone();

        let is_old_password_valid = tokio::task::spawn_blocking(move || {
            verify_password(&old_password, &user_password_hash)
        })
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Thread join error during password verification: {}", e)))??;


        if !is_old_password_valid {
            return Err(AppError::InvalidCredentials("Invalid old password".to_string()));
        }

        let new_password = req.new_password.clone();
        let new_password_hash = tokio::task::spawn_blocking(move || {
            crate::utils::crypto::hash_password(&new_password)
        })
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Thread join error during password hashing: {}", e)))??;

        self.user_repo
            .update_user_password(user_id, &new_password_hash)
            .await?;

        info!(user_id = user_id, "User password updated successfully");
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use std::sync::Arc;
 
//     use async_trait::async_trait;
//     use chrono::Utc;
//     use mockall::mock;
 
//     use crate::{
//         dtos::{auth_dto::UpdatePasswordRequest, user_dto::UpdateUserRequest},
//         errors::AppError,
//         models::users::{User, UserWithRoles}, // adjust path/name to your actual model
//         repositories::AuthRepositoryTrait,
//         services::auth_service::{AuthService, AuthServiceTrait},
//     };
 
//     // ── Mock repository ───────────────────────────────────────────────────────
//     // Mirrors AuthRepositoryTrait exactly — update method signatures here
//     // if your trait changes.
//     mock! {
//         AuthRepo {}
 
//         #[async_trait]
//         impl AuthRepositoryTrait for AuthRepo {
//             async fn find_user_by_email(&self, email: &str) -> AppResult<Option<UserWithRoles>>;
//             async fn find_user_by_id(&self, id: i64) -> AppResult<Option<UserWithRoles>>;
//             async fn check_email_exists(&self, email: &str) -> AppResult<bool>;
//             async fn update_user(
//                 &self,
//                 id: i64,
//                 name: Option<&str>,
//                 email: Option<&str>,
//                 phone: Option<&str>,
//             ) -> AppResult<()>;
//             async fn update_user_photo(&self, user_id: i64, photo_url: &str) -> AppResult<()>;
//             async fn clear_user_photo(&self, user_id: i64) -> AppResult<()>;
//             async fn update_user_password(&self, user_id: i64, password_hash: &str) -> AppResult<()>;
//         }
//     }
 
//     use crate::errors::AppResult;
 
//     // ── Fixtures ──────────────────────────────────────────────────────────────
 
//     fn fake_user(id: i64, email: &str) -> UserWithRoles {
//         UserWithRoles {
//             id,
//             name: "Test User".to_string(),
//             email: email.to_string(),
//             password: "$argon2id$v=19$m=19456,t=2,p=1$fakehash".to_string(),
//             photo: None,
//             phone: None,
//             roles: Some(vec!["STAFF".to_string()]),
//             deleted_at: None,
//             created_at: Utc::now(),
//             updated_at: Utc::now(),
//         }
//     }
 
//     // ── get_profile ───────────────────────────────────────────────────────────
 
//     #[tokio::test]
//     async fn test_get_profile_returns_user_when_found() {
//         let mut mock = MockAuthRepo::new();
//         mock.expect_find_user_by_id()
//             .withf(|id| *id == 1)
//             .returning(|_| Ok(Some(fake_user(1, "test@example.com"))));
 
//         let service = build_service(mock);
 
//         let result = service.get_profile(1).await;
 
//         assert!(result.is_ok());
//         let profile = result.unwrap();
//         assert_eq!(profile.id, 1);
//         assert_eq!(profile.email, "test@example.com");
//     }
 
//     #[tokio::test]
//     async fn test_get_profile_returns_not_found_when_missing() {
//         let mut mock = MockAuthRepo::new();
//         mock.expect_find_user_by_id()
//             .returning(|_| Ok(None));
 
//         let service = build_service(mock);
 
//         let result = service.get_profile(999).await;
 
//         assert!(matches!(result, Err(AppError::NotFound(_))));
//     }
 
//     #[tokio::test]
//     async fn test_get_profile_response_never_contains_password() {
//         let mut mock = MockAuthRepo::new();
//         mock.expect_find_user_by_id()
//             .returning(|_| Ok(Some(fake_user(1, "test@example.com"))));
 
//         let service = build_service(mock);
 
//         let profile = service.get_profile(1).await.unwrap();
 
//         // UserResponse should not have a password field at all — this is
//         // a compile-time guarantee via the type system, but we assert
//         // here as documentation of that guarantee.
//         let serialized = serde_json::to_string(&profile).unwrap();
//         assert!(!serialized.contains("password"));
//         assert!(!serialized.contains("argon2"));
//     }
 
//     // ── update_profile ────────────────────────────────────────────────────────
 
//     #[tokio::test]
//     async fn test_update_profile_success() {
//         let mut mock = MockAuthRepo::new();
//         mock.expect_find_user_by_id()
//             .returning(|_| Ok(Some(fake_user(1, "old@example.com"))));
//         mock.expect_check_email_exists()
//             .returning(|_| Ok(false));
//         mock.expect_update_user()
//             .returning(|_, _, _, _| Ok(()));
 
//         let service = build_service(mock);
 
//         let result = service
//             .update_profile(
//                 1,
//                 UpdateUserRequest {
//                     name: Some("New Name".to_string()),
//                     email: Some("new@example.com".to_string()),
//                     phone: None,
//                 },
//             )
//             .await;
 
//         assert!(result.is_ok());
//     }
 
//     #[tokio::test]
//     async fn test_update_profile_nonexistent_user_returns_not_found() {
//         let mut mock = MockAuthRepo::new();
//         mock.expect_find_user_by_id()
//             .returning(|_| Ok(None));
 
//         let service = build_service(mock);
 
//         let result = service
//             .update_profile(
//                 999,
//                 UpdateUserRequest {
//                     name: Some("New Name".to_string()),
//                     email: None,
//                     phone: None,
//                 },
//             )
//             .await;
 
//         assert!(matches!(result, Err(AppError::NotFound(_))));
//     }
 
//     #[tokio::test]
//     async fn test_update_profile_duplicate_email_returns_conflict() {
//         let mut mock = MockAuthRepo::new();
//         mock.expect_find_user_by_id()
//             .returning(|_| Ok(Some(fake_user(1, "old@example.com"))));
//         mock.expect_check_email_exists()
//             .returning(|_| Ok(true)); // email already taken by someone else
 
//         let service = build_service(mock);
 
//         let result = service
//             .update_profile(
//                 1,
//                 UpdateUserRequest {
//                     name: None,
//                     email: Some("taken@example.com".to_string()),
//                     phone: None,
//                 },
//             )
//             .await;
 
//         assert!(matches!(result, Err(AppError::Conflict(_))));
//     }
 
//     #[tokio::test]
//     async fn test_update_profile_same_email_does_not_trigger_conflict_check() {
//         // Edge case: if user submits their OWN current email unchanged,
//         // check_email_exists would find it (since it belongs to them) and
//         // incorrectly reject the update. This test documents the current
//         // behavior — see note below the test for the recommended fix.
//         let mut mock = MockAuthRepo::new();
//         mock.expect_find_user_by_id()
//             .returning(|_| Ok(Some(fake_user(1, "same@example.com"))));
//         mock.expect_check_email_exists()
//             .returning(|_| Ok(true)); // exists because it's THEIR OWN row
//         // update_user is intentionally NOT mocked to expect a call here —
//         // if the current implementation calls it, mockall will panic on
//         // an unexpected call, surfacing the bug.
 
//         let service = build_service(mock);
 
//         let result = service
//             .update_profile(
//                 1,
//                 UpdateUserRequest {
//                     name: None,
//                     email: Some("same@example.com".to_string()),
//                     phone: None,
//                 },
//             )
//             .await;
 
//         // Current implementation will incorrectly return Conflict here.
//         // Recommended fix: exclude the user's own ID in check_email_exists,
//         // similar to the `exclude_id` pattern used in WarehouseRepository.
//         assert!(matches!(result, Err(AppError::Conflict(_))));
//     }
 
//     // ── update_profile_photo ──────────────────────────────────────────────────
 
//     #[tokio::test]
//     async fn test_update_profile_photo_success() {
//         let mut mock = MockAuthRepo::new();
//         mock.expect_find_user_by_id()
//             .returning(|_| Ok(Some(fake_user(1, "test@example.com"))));
//         mock.expect_update_user_photo()
//             .withf(|id, url| *id == 1 && url == "https://cdn.example.com/photo.jpg")
//             .returning(|_, _| Ok(()));
 
//         let service = build_service(mock);
 
//         let result = service
//             .update_profile_photo(1, "https://cdn.example.com/photo.jpg")
//             .await;
 
//         assert!(result.is_ok());
//     }
 
//     #[tokio::test]
//     async fn test_update_profile_photo_nonexistent_user_returns_not_found() {
//         let mut mock = MockAuthRepo::new();
//         mock.expect_find_user_by_id()
//             .returning(|_| Ok(None));
 
//         let service = build_service(mock);
 
//         let result = service
//             .update_profile_photo(999, "https://cdn.example.com/photo.jpg")
//             .await;
 
//         assert!(matches!(result, Err(AppError::NotFound(_))));
//     }
 
//     // ── delete_profile_photo ──────────────────────────────────────────────────
 
//     #[tokio::test]
//     async fn test_delete_profile_photo_success() {
//         let mut mock = MockAuthRepo::new();
//         mock.expect_find_user_by_id()
//             .returning(|_| Ok(Some(fake_user(1, "test@example.com"))));
//         mock.expect_clear_user_photo()
//             .withf(|id| *id == 1)
//             .returning(|_| Ok(()));
 
//         let service = build_service(mock);
 
//         let result = service.delete_profile_photo(1).await;
 
//         assert!(result.is_ok());
//     }
 
//     #[tokio::test]
//     async fn test_delete_profile_photo_nonexistent_user_returns_not_found() {
//         let mut mock = MockAuthRepo::new();
//         mock.expect_find_user_by_id()
//             .returning(|_| Ok(None));
 
//         let service = build_service(mock);
 
//         let result = service.delete_profile_photo(999).await;
 
//         assert!(matches!(result, Err(AppError::NotFound(_))));
//     }
 
//     // ── update_profile_password ───────────────────────────────────────────────
 
//     #[tokio::test]
//     async fn test_update_profile_password_success() {
//         let mut mock = MockAuthRepo::new();
//         mock.expect_find_user_by_id()
//             .returning(|_| Ok(Some(fake_user(1, "test@example.com"))));
//         mock.expect_update_user_password()
//             .returning(|_, _| Ok(()));
 
//         let service = build_service(mock);
 
//         let result = service
//             .update_profile_password(
//                 1,
//                 UpdatePasswordRequest {
//                     password: "NewPassword123".to_string(),
//                 },
//             )
//             .await;
 
//         assert!(result.is_ok());
//     }
 
//     #[tokio::test]
//     async fn test_update_profile_password_nonexistent_user_returns_not_found() {
//         let mut mock = MockAuthRepo::new();
//         mock.expect_find_user_by_id()
//             .returning(|_| Ok(None));
 
//         let service = build_service(mock);
 
//         let result = service
//             .update_profile_password(
//                 999,
//                 UpdatePasswordRequest {
//                     password: "NewPassword123".to_string(),
//                 },
//             )
//             .await;
 
//         assert!(matches!(result, Err(AppError::NotFound(_))));
//     }
 
//     // NOTE: `update_profile_password` currently stores `req.password` as-is
//     // via `update_user_password`. If hashing is expected to happen inside
//     // the service (consistent with how `register`/`login` hash elsewhere),
//     // double check that `update_user_password` callers hash before calling,
//     // or that the repository itself hashes — whichever it is, write a test
//     // asserting the stored value is NOT the plaintext password, mirroring
//     // `test_register_password_is_stored_as_hash_not_plaintext` from the
//     // original AuthService tests.
 
//     // ── Test helper ───────────────────────────────────────────────────────────
 
//     fn build_service(mock: MockAuthRepo) -> AuthService<MockAuthRepo> {
//         let config = Arc::new(crate::infrastructure::config::Config {
//             // Fill with whatever minimal fields your Config requires.
//             // Only `auth.*` fields matter for AuthService's non-Redis paths.
//             ..Default::default()
//         });
 
//         // login/refresh/logout need a real Redis pool and are NOT covered
//         // by these unit tests — see module doc comment above. For tests
//         // that only exercise get_profile/update_profile/*, a Redis pool
//         // is still required to construct AuthService, but it is never
//         // actually called by those methods, so a lazily-connected pool
//         // (never queried) is safe here.
//         let redis = Arc::new(
//             deadpool_redis::Config::from_url("redis://127.0.0.1:0") // unused
//                 .create_pool(Some(deadpool_redis::Runtime::Tokio1))
//                 .expect("pool creation is lazy, does not connect"),
//         );
 
//         AuthService::new(Arc::new(mock), config, redis)
//     }
// }