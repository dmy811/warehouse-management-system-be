use async_trait::async_trait;
use sqlx::PgPool;

use crate::{constants::roles, errors::{AppError, AppResult}, models::User};

// pub async fn create_user(pool: &DBPool, payload: &RegisterRequest) -> sqlx::Result<User, sqlx::Error>{
//     let mut tx = pool.begin().await?;
//     let user = sqlx::query_as!(
//         User,
//         r#"
//         INSERT INTO public.users (name, email, password, photo, phone)
//         VALUES ($1, $2, $3, $4, $5)
//         RETURNING id, name, email, password, photo, phone, created_at, updated_at
//         "#,
//         payload.name,
//         payload.email,
//         payload.password,
//         payload.photo,
//         payload.phone
//     ).fetch_one(&mut *tx).await?;

//     sqlx::query!(
//         r#"
//         INSERT INTO public.user_roles (user_id, role_id)
//         VALUES ($1, (SELECT id FROM public.roles WHERE name = $2))
//         "#,
//         user.id,
//         role_name
//     ).execute(&mut *tx).await?;

//     tx.commit().await?;
//     Ok(user)
// }

#[async_trait]
pub trait UserRepositoryTrait: Send + Sync {
    async fn create(&self, name: &str, email: &str, password_hash: &str, phone: Option<&str>, role: &str) -> AppResult<User>;
}

pub struct UserRepository {
    db: PgPool
}

impl UserRepository {
    pub fn new(db: PgPool) -> Self {
        Self {
            db
        }
    }
}

#[async_trait]
impl UserRepositoryTrait for UserRepository {
    async fn create(
        &self,
        name: &str,
        email: &str,
        password_hash: &str,
        phone: Option<&str>,
        role: &str
    ) -> AppResult<User>{
        let mut tx = self.db.begin().await?;

        let role_exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM roles WHERE name = $1)",
            role
        )
        .fetch_one(tx.as_mut())
        .await?
        .unwrap_or(false);

        if !role_exists {
            return Err(AppError::Validation(
                format!("Role '{}' does not exist", role)
            ))
        }

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
        .fetch_one(tx.as_mut()) // cara lama (&mut *tx)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO user_roles (user_id, role_id)
            SELECT $1, r.id FROM roles r WHERE r.name = $2
            ON CONFLICT (user_id, role_id) DO NOTHING
            "#,
            user.id,
            role
        )
        .execute(tx.as_mut())
        .await?;

        tx.commit().await?;
    
        Ok(user)
    }
}