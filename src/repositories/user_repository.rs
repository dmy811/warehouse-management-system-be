use async_trait::async_trait;
use sqlx::PgPool;

use crate::{errors::{AppError, AppResult}, models::{User, UserWithRole}, response::ListQuery};
#[async_trait]
pub trait UserRepositoryTrait: Send + Sync {
    async fn create(&self, name: &str, email: &str, password_hash: &str, phone: Option<&str>, role: &str) -> AppResult<User>;
    async fn find_all(&self, query: &ListQuery) -> AppResult<(Vec<UserWithRole>, i64)>;
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

    async fn find_all(&self, query: &ListQuery) -> AppResult<(Vec<UserWithRole>, i64)> {
        let search_pattern = query
            .search
            .as_ref()
            .map(|s| format!("%{}%", s.to_lowercase()));

        let sort_col = query.sort_column();
        let sort_dir = query.sort_direction();

        let sql = format!(
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
            WHERE u.deleted_at IS NULL
                AND ($1::TEXT IS NULL OR (
                    LOWER(u.name) LIKE $1
                OR  LOWER(u.email) LIKE $1
                ))
            ORDER BY {sort_col} {sort_dir}
            LIMIT $2
            OFFSET $3
            "#
        );

        let items = sqlx::query_as::<_, UserWithRole>(&sql)
            .bind(&search_pattern)
            .bind(query.per_page())
            .bind(query.offset())
            .fetch_all(&self.db)
            .await?;

        let count_sql = r#"
            SELECT COUNT(*)
            FROM users u
            WHERE u.deleted_at IS NULL
              AND ($1::TEXT IS NULL OR (
                    LOWER(u.name) LIKE $1
              OR  LOWER(u.email) LIKE $1
              ))
        "#;
 
        let total: i64 = sqlx::query_scalar(count_sql)
            .bind(&search_pattern)
            .fetch_one(&self.db)
            .await?;
 
        Ok((items, total))
    }
}