use async_trait::async_trait;
use sqlx::PgPool;
use crate::{errors::AppResult, models::{User, UserWithRole}};

// -- Send artinya Boleh dipindahkan antar thread
// -- Sync artinya Boleh diakses dari banyak thread secara bersamaan (via reference)
// -- Kenapa butuh kedua itu, karena gua pakai Arc<dyn AuthRepositoryTrait> sebagai dependency injection, karena dyn trait tidak otomatis Send dan Sync
// -- Arc<dyn AuthRepositoryTrait> artinya = 
// -- bisa berbagai implementasi (dyn)
// -- bisa dishare ke banyak thread (Arc)
// -- aman untuk async (Send + Sync)
#[async_trait]
pub trait AuthRepositoryTrait: Send + Sync {
    async fn find_by_email(&self, email: &str) -> AppResult<Option<UserWithRole>>;
    async fn find_by_id(&self, id: i32) -> AppResult<Option<UserWithRole>>;
    async fn email_exists(&self, email: &str) -> AppResult<bool>;
    async fn create(&self, name: &str, email: &str, password_hash: &str, phone: Option<&str>) -> AppResult<User>;
    async fn update_photo(&self, user_id: i32, photo_url: &str) -> AppResult<()>;
    async fn clear_photo(&self, user_id: i32) -> AppResult<()>;
}

pub struct AuthRepository {
    db: PgPool
}

impl AuthRepository {
    pub fn new(db: PgPool) -> Self {
        Self {
            db
        }
    }
}