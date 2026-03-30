use std::sync::Arc;

use argon2::password_hash;
use async_trait::async_trait;
use tracing::{info, warn};

use crate::{dtos::{AuthResponse, LoginRequest, RegisterRequest, UserResponse}, errors::{AppError, AppResult}, infrastructure::config::Config, repositories::AuthRepositoryTrait, utils::{crypto::{hash_password, verify_password}, jwt::create_token}};

#[async_trait]
pub trait AuthServiceTrait: Send + Sync {
    async fn register(&self, req: RegisterRequest) -> AppResult<AuthResponse>;
    async fn login(&self, req: LoginRequest) -> AppResult<AuthResponse>;
    async fn me(&self, user_id: i64) -> AppResult<UserResponse>;
    async fn update_photo(&self, user_id: i64, photo_url: &str) -> AppResult<()>;
    async fn delete_photo(&self, user_id: i64) -> AppResult<()>;
}

pub struct AuthService<R: AuthRepositoryTrait> {
    repo: Arc<R>,
    config: Arc<Config>
}

impl<R: AuthRepositoryTrait> AuthService<R> {
    pub fn new(repo: Arc<R>, config: Arc<Config>) -> Self {
        Self {
            repo,
            config
        }
    }
}

#[async_trait]
impl<R: AuthRepositoryTrait> AuthServiceTrait for AuthService<R> {
    async fn register(&self, req: RegisterRequest) -> AppResult<AuthResponse>{
        if self.repo.email_exists(&req.email).await? {
            return Err(AppError::Conflict(format!(
                "Email '{}' is already registered",
                req.email
            )))
        }

        let password_hash = tokio::task::spawn_blocking({
            let password = req.password.clone();
            move || hash_password(&password)
        })
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Thread join error: {}", e)))??;

        let user = self
            .repo
            .create(&req.name, &req.email, &password_hash, req.phone.as_deref())
            .await?;

        info!(user_id = user.id, email = %req.email, "New user registered");

        let user_with_role = self
            .repo
            .find_by_id(user.id)
            .await?
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("User not found after insert")))?;

        let role = user_with_role.role_name.clone().unwrap_or_default();

        let token = create_token(
            user.id,
            &role,
            &self.config.jwt_secret,
            self.config.jwt_expires_in_secs
        )?;

        Ok(AuthResponse::new(token, user_with_role))
    }

    // 🧠 1. Prinsip utama tokio::join!

    // 👉 join! hanya bisa dipakai kalau:

    // semua operasi itu independen (tidak saling bergantung)

    // 🔍 2. Analisis kode kamu
    // ❌ Bagian ini TIDAK bisa di-join
    // if self.repo.email_exists(&req.email).await? {
    //     return Err(...);
    // }

    // Kenapa?

    // ini validasi awal
    // kalau email sudah ada → stop

    // 👉 tidak bisa diparalelkan dengan yang lain

    // ❌ Ini juga tidak bisa di-join
    // let user = self.repo.create(...).await?;

    // Kenapa?

    // butuh password_hash dulu
    // berarti bergantung hasil sebelumnya
    // ❌ Ini juga tidak bisa di-join
    // let user_with_role = self.repo.find_by_id(user.id).await?;

    // Kenapa?

    // butuh user.id dari create
    // lagi-lagi dependent
    // ⚡ Jadi alurnya:
    // email_exists
    // ↓
    // hash_password
    // ↓
    // create_user
    // ↓
    // find_by_id
    // ↓
    // create_token

    // 👉 Semua ini chain / berurutan (dependent)
    // 👉 ❗ Tidak bisa di-parallel-kan

    // 🔥 3. Jadi di mana join! bisa dipakai?
    // ✅ Contoh yang BENAR

    // Misalnya kamu punya:

    // let user = repo.get_user(id);
    // let orders = repo.get_orders(id);

    // 👉 ini bisa:

    // let (user, orders) = tokio::join!(user, orders);

    // Karena:

    // tidak saling bergantung

    async fn login(&self, req: LoginRequest) -> AppResult<AuthResponse>{
        let user = self
            .repo
            .find_by_email(&req.email)
            .await?
            .ok_or_else(|| AppError::InvalidCredentials("Email doesn't exists!".to_string()))?;

        let password_hash = user.password.clone();
        let password = req.password.clone();
        let is_valid = tokio::task::spawn_blocking(move || verify_password(&password, &password_hash))
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Thread join error: {}", e)))??;

        if !is_valid {
            warn!(email = %req.email, "Failed login attempt - wrong password");
            return Err(AppError::InvalidCredentials("Password doesn't match!".to_string()));
        }

        info!(user_id = user.id, "User logged in");
        let role = user.role_name.clone().unwrap_or_default();
        let token = create_token(
            user.id,
            &role,
            &self.config.jwt_secret,
            self.config.jwt_expires_in_secs
        )?;

        Ok(AuthResponse::new(token, user))
    }

    async fn me(&self, user_id: i64) -> AppResult<UserResponse>{
        let user = self
            .repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User".to_string()))?;

        Ok(UserResponse::from(user))
    }

    async fn update_photo(&self, user_id: i64, photo_url: &str) -> AppResult<()>{
        self.repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User".to_string()))?;
        
        self.repo.update_photo(user_id, photo_url).await
    }

    async fn delete_photo(&self, user_id: i64) -> AppResult<()> {
        self.repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User".to_string()))?;
 
        self.repo.clear_photo(user_id).await
    }
}


// #[cfg(test)]
// mod tests {
//     use super::*;
//     use chrono::Utc;
//     use mockall::mock;
 
//     use crate::{
//         errors::AppError,
//         models::{User, UserWithRole},
//         utils::crypto::hash_password,
//     };
 
//     // ── Mock repository ───────────────────────────────────────────────────────
//     // mockall generates a MockAuthRepository that implements AuthRepositoryTrait.
//     // Each test configures only the methods it needs via .expect_*().
 
//     mock! {
//         AuthRepo {}
 
//         #[async_trait]
//         impl AuthRepositoryTrait for AuthRepo {
//             async fn find_by_email(&self, email: &str) -> AppResult<Option<UserWithRole>>;
//             async fn find_by_id(&self, id: i32) -> AppResult<Option<UserWithRole>>;
//             async fn email_exists(&self, email: &str) -> AppResult<bool>;
//             async fn create(
//                 &self,
//                 name: &str,
//                 email: &str,
//                 password_hash: &str,
//                 phone: Option<&str>,
//             ) -> AppResult<User>;
//             async fn update_photo(&self, user_id: i32, photo_url: &str) -> AppResult<()>;
//             async fn clear_photo(&self, user_id: i32) -> AppResult<()>;
//         }
//     }
 
//     // ── Test fixtures ─────────────────────────────────────────────────────────
 
//     fn test_config() -> Arc<Config> {
//         Arc::new(Config {
//             database_url: String::new(),
//             jwt_secret: "test_secret_key_long_enough_32ch".to_string(),
//             jwt_expires_in_secs: 3600,
//             app_env: crate::infrastructure::config::AppEnv::Development,
//             cloudinary: crate::infrastructure::config::CloudinaryConfig {
//                 cloud_name: String::new(),
//                 api_key: String::new(),
//                 api_secret: String::new(),
//             },
//         })
//     }
 
//     fn fake_user() -> User {
//         User {
//             id: 1,
//             name: "Test User".to_string(),
//             email: "test@example.com".to_string(),
//             password: hash_password("Password123").unwrap(),
//             photo: None,
//             phone: None,
//             deleted_at: None,
//             created_at: Utc::now(),
//             updated_at: Utc::now(),
//         }
//     }
 
//     fn fake_user_with_role(role: Option<&str>) -> UserWithRole {
//         UserWithRole {
//             id: 1,
//             name: "Test User".to_string(),
//             email: "test@example.com".to_string(),
//             password: hash_password("Password123").unwrap(),
//             photo: None,
//             phone: None,
//             role_name: role.map(String::from),
//             deleted_at: None,
//             created_at: Utc::now(),
//             updated_at: Utc::now(),
//         }
//     }
 
//     // ── Login tests ───────────────────────────────────────────────────────────
 
//     #[tokio::test]
//     async fn test_login_success_returns_token_and_user() {
//         let mut mock = MockAuthRepo::new();
//         mock.expect_find_by_email()
//             .returning(|_| Ok(Some(fake_user_with_role(Some("ADMIN")))));
 
//         let service = AuthService::new(Arc::new(mock), test_config());
//         let result = service.login(LoginRequest {
//             email: "test@example.com".to_string(),
//             password: "Password123".to_string(),
//         }).await;
 
//         assert!(result.is_ok());
//         let auth = result.unwrap();
//         assert!(!auth.access_token.is_empty());
//         assert_eq!(auth.token_type, "Bearer");
//         assert_eq!(auth.user.email, "test@example.com");
//     }
 
//     #[tokio::test]
//     async fn test_login_nonexistent_email_returns_invalid_credentials() {
//         let mut mock = MockAuthRepo::new();
//         // User not found — returns None
//         mock.expect_find_by_email().returning(|_| Ok(None));
 
//         let service = AuthService::new(Arc::new(mock), test_config());
//         let result = service.login(LoginRequest {
//             email: "ghost@example.com".to_string(),
//             password: "Password123".to_string(),
//         }).await;
 
//         // Must NOT distinguish "user not found" from "wrong password"
//         // to prevent user enumeration attacks
//         assert!(matches!(result, Err(AppError::InvalidCredentials)));
//     }
 
//     #[tokio::test]
//     async fn test_login_wrong_password_returns_invalid_credentials() {
//         let mut mock = MockAuthRepo::new();
//         mock.expect_find_by_email()
//             .returning(|_| Ok(Some(fake_user_with_role(None))));
 
//         let service = AuthService::new(Arc::new(mock), test_config());
//         let result = service.login(LoginRequest {
//             email: "test@example.com".to_string(),
//             password: "WrongPassword".to_string(),
//         }).await;
 
//         assert!(matches!(result, Err(AppError::InvalidCredentials)));
//     }
 
//     #[tokio::test]
//     async fn test_login_error_message_is_identical_for_wrong_email_and_wrong_password() {
//         // Security: both failure modes must return exactly the same error
//         // so an attacker cannot tell which one failed
//         let mut mock_no_user = MockAuthRepo::new();
//         mock_no_user.expect_find_by_email().returning(|_| Ok(None));
 
//         let mut mock_wrong_pw = MockAuthRepo::new();
//         mock_wrong_pw.expect_find_by_email()
//             .returning(|_| Ok(Some(fake_user_with_role(None))));
 
//         let service_no_user = AuthService::new(Arc::new(mock_no_user), test_config());
//         let service_wrong_pw = AuthService::new(Arc::new(mock_wrong_pw), test_config());
 
//         let req = LoginRequest {
//             email: "test@example.com".to_string(),
//             password: "Wrong".to_string(),
//         };
 
//         let err1 = service_no_user.login(req.clone()).await.unwrap_err();
//         let err2 = service_wrong_pw.login(LoginRequest {
//             email: "test@example.com".to_string(),
//             password: "Wrong".to_string(),
//         }).await.unwrap_err();
 
//         // Both errors must be the same variant
//         assert!(matches!(err1, AppError::InvalidCredentials));
//         assert!(matches!(err2, AppError::InvalidCredentials));
//         // Both errors must have the same message
//         assert_eq!(err1.to_string(), err2.to_string());
//     }
 
//     // ── Register tests ────────────────────────────────────────────────────────
 
//     #[tokio::test]
//     async fn test_register_success_returns_token_and_user() {
//         let mut mock = MockAuthRepo::new();
//         mock.expect_email_exists().returning(|_| Ok(false));
//         mock.expect_create().returning(|_, _, _, _| Ok(fake_user()));
//         mock.expect_find_by_id()
//             .returning(|_| Ok(Some(fake_user_with_role(None))));
 
//         let service = AuthService::new(Arc::new(mock), test_config());
//         let result = service.register(RegisterRequest {
//             name: "Test User".to_string(),
//             email: "new@example.com".to_string(),
//             password: "Password123".to_string(),
//             phone: None,
//         }).await;
 
//         assert!(result.is_ok());
//         let auth = result.unwrap();
//         assert!(!auth.access_token.is_empty());
//     }
 
//     #[tokio::test]
//     async fn test_register_duplicate_email_returns_conflict() {
//         let mut mock = MockAuthRepo::new();
//         // Email already exists
//         mock.expect_email_exists().returning(|_| Ok(true));
 
//         let service = AuthService::new(Arc::new(mock), test_config());
//         let result = service.register(RegisterRequest {
//             name: "Test User".to_string(),
//             email: "existing@example.com".to_string(),
//             password: "Password123".to_string(),
//             phone: None,
//         }).await;
 
//         assert!(matches!(result, Err(AppError::Conflict(_))));
//     }
 
//     #[tokio::test]
//     async fn test_register_password_is_stored_as_hash_not_plaintext() {
//         let mut mock = MockAuthRepo::new();
//         mock.expect_email_exists().returning(|_| Ok(false));
 
//         // Capture the password_hash argument passed to create()
//         mock.expect_create()
//             .withf(|_, _, password_hash, _| {
//                 // Must start with argon2 prefix, NOT be plaintext
//                 password_hash.starts_with("$argon2") && *password_hash != "Password123"
//             })
//             .returning(|_, _, _, _| Ok(fake_user()));
 
//         mock.expect_find_by_id()
//             .returning(|_| Ok(Some(fake_user_with_role(None))));
 
//         let service = AuthService::new(Arc::new(mock), test_config());
//         let result = service.register(RegisterRequest {
//             name: "Test User".to_string(),
//             email: "new@example.com".to_string(),
//             password: "Password123".to_string(),
//             phone: None,
//         }).await;
 
//         // If withf() assertion failed, create() would not have been called
//         // and this would return an error from mockall
//         assert!(result.is_ok(), "Password was not hashed correctly: {:?}", result.err());
//     }
 
//     // ── Me tests ──────────────────────────────────────────────────────────────
 
//     #[tokio::test]
//     async fn test_me_returns_user_without_password() {
//         let mut mock = MockAuthRepo::new();
//         mock.expect_find_by_id()
//             .with(mockall::predicate::eq(1))
//             .returning(|_| Ok(Some(fake_user_with_role(Some("STAFF")))));
 
//         let service = AuthService::new(Arc::new(mock), test_config());
//         let result = service.me(1).await;
 
//         assert!(result.is_ok());
//         let user = result.unwrap();
//         assert_eq!(user.id, 1);
//         assert_eq!(user.email, "test@example.com");
//         assert_eq!(user.role, Some("STAFF".to_string()));
//         // UserResponse has no password field — confirmed by type system
//     }
 
//     #[tokio::test]
//     async fn test_me_nonexistent_user_returns_not_found() {
//         let mut mock = MockAuthRepo::new();
//         mock.expect_find_by_id().returning(|_| Ok(None));
 
//         let service = AuthService::new(Arc::new(mock), test_config());
//         let result = service.me(999).await;
 
//         assert!(matches!(result, Err(AppError::NotFound(_))));
//     }
// }