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
    async fn find_by_id(&self, id: i64) -> AppResult<Option<UserWithRole>>;
    async fn email_exists(&self, email: &str) -> AppResult<bool>;
    async fn create(&self, name: &str, email: &str, password_hash: &str, phone: Option<&str>) -> AppResult<User>;
    async fn update_photo(&self, user_id: i64, photo_url: &str) -> AppResult<()>;
    async fn clear_photo(&self, user_id: i64) -> AppResult<()>;
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

#[async_trait]
impl AuthRepositoryTrait for AuthRepository {
    async fn find_by_email(&self, email: &str) -> AppResult<Option<UserWithRole>> {
        let user = sqlx::query_as!(
            UserWithRole,
            r#"
            SELECT
                u.id,
                u.name,
                u.email,
                u.password,
                u.photo,
                u.phone,
                u.deleted_at,
                u.created_at,
                u.updated_at,
                r.name as role_name
            FROM users u
            LEFT JOIN user_roles ur ON ur.user_id = u.id
            LEFT JOIN roles r ON r.id = ur.role_id
            WHERE u.email = $1
                AND u.deleted_at IS NULL
            LIMIT 1
            "#,
            email
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(user)
    }

    async fn find_by_id(&self, id: i64) -> AppResult<Option<UserWithRole>>{
        let user = sqlx::query_as!(
            UserWithRole,
            r#"
            SELECT
                u.id,
                u.name,
                u.email,
                u.password,
                u.photo,
                u.phone,
                u.deleted_at,
                u.created_at,
                u.updated_at,
                r.name as role_name
            FROM users u
            LEFT JOIN user_roles ur ON ur.user_id = u.id
            LEFT JOIN roles r ON r.id = ur.role_id
            WHERE u.id = $1
                AND u.deleted_at IS NULL
            LIMIT 1
            "#,
            id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(user)
    }

    async fn email_exists(&self, email: &str) -> AppResult<bool>{
        let exists = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM users WHERE email = 
            $1 AND deleted_at IS NULL)"#,
            email
        )
        .fetch_one(&self.db)
        .await?
        .unwrap_or(false);

        Ok(exists)
    }

    async fn create(
        &self,
        name: &str,
        email: &str,
        password_hash: &str,
        phone: Option<&str>
    ) -> AppResult<User> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (name, email, password, phone)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
            name,
            email,
            password_hash,
            phone
        )
        .fetch_one(&self.db)
        .await?;

        Ok(user)
    }

    async fn update_photo(&self, user_id: i64, photo_url: &str) -> AppResult<()>{
        sqlx::query!(
            r#"
            UPDATE users SET photo = $2, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            user_id,
            photo_url
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    async fn clear_photo(&self, user_id: i64) -> AppResult<()>{
        sqlx::query!(
            r#"
            UPDATE users SET photo = NULL, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            user_id
        )
        .execute(&self.db)
        .await?;
    
        Ok(())
    }
}