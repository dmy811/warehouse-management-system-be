use async_trait::async_trait;
use sqlx::PgPool;

use crate::{errors::{AppError, AppResult}, models::{User, UserWithRole}, response::ListQuery};
#[async_trait]
pub trait UserRepositoryTrait: Send + Sync {
    async fn check_email_exists(&self, email: &str) -> AppResult<bool>;
    async fn create_user(&self, name: &str, email: &str, password_hash: &str, phone: Option<&str>, role: &str) -> AppResult<User>;
    async fn find_all_users(&self, query: &ListQuery) -> AppResult<(Vec<UserWithRole>, i64)>;
    async fn find_user_by_id(&self, user_id: i64) -> AppResult<Option<UserWithRole>>;
    async fn update_user(&self, user_id: i64, name: Option<&str>, email: Option<&str>, phone: Option<&str>) -> AppResult<Option<User>>;
    async fn user_soft_delete(&self, user_id: i64) -> AppResult<bool>;
    async fn user_hard_delete(&self, user_id: i64) -> AppResult<bool>;
    async fn add_role(&self, user_id: i64, role: &str) -> AppResult<bool>;
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
    async fn check_email_exists(&self, email: &str) -> AppResult<bool>{
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


    async fn create_user(
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

    async fn find_all_users(&self, query: &ListQuery) -> AppResult<(Vec<UserWithRole>, i64)> {
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
                ARRAY_AGG(r.name) FILTER (WHERE r.name IS NOT NULL) as "roles!"
            FROM users u
            LEFT JOIN user_roles ur ON ur.user_id = u.id
            LEFT JOIN roles r ON r.id = ur.role_id
            WHERE u.deleted_at IS NULL
                AND ($1::TEXT IS NULL OR (LOWER(u.name) LIKE $1 OR LOWER(u.email) LIKE $1))
            GROUP BY u.id
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
              AND ($1::TEXT IS NULL OR (LOWER(u.name) LIKE $1 OR LOWER(u.email) LIKE $1))
        "#;
 
        let total: i64 = sqlx::query_scalar(count_sql)
            .bind(&search_pattern)
            .fetch_one(&self.db)
            .await?;
 
        Ok((items, total))
    }

    async fn find_user_by_id(&self, user_id: i64) -> AppResult<Option<UserWithRole>> {
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
                ARRAY_AGG(r.name) FILTER (WHERE r.name IS NOT NULL) as "roles!"
            FROM users u
            LEFT JOIN user_roles ur ON ur.user_id = u.id
            LEFT JOIN roles r ON r.id = ur.role_id
            WHERE u.id = $1
                AND u.deleted_at IS NULL
            GROUP BY u.id
            LIMIT 1
            "#,
            user_id
        )
        .fetch_optional(&self.db)
        .await?;

    Ok(user)
    }

    async fn update_user(&self, user_id: i64, name: Option<&str>, email: Option<&str>, phone: Option<&str>) -> AppResult<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users SET
            name = COALESCE($2, name),
            email = COALESCE($3, email),
            phone = COALESCE($4, phone)
            WHERE id = $1
            AND deleted_at IS NULL
            RETURNING *
            "#,
            user_id,
            name,
            email,
            phone
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(user)
    }

    async fn user_soft_delete(&self, user_id: i64) -> AppResult<bool> {
        let result = sqlx::query!(
            r#"
            UPDATE users
            SET deleted_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            user_id
        )
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn user_hard_delete(&self, user_id: i64) -> AppResult<bool> {
        let result = sqlx::query!(
            r#"
            DELETE FROM users WHERE id = $1
            "#,
            user_id
        )
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn add_role(&self, user_id: i64, role: &str) -> AppResult<bool> {
        let role_id: i64 = sqlx::query_scalar!(
            r#"
            SELECT id
            FROM public.roles
            WHERE name = $1
            "#,
            role
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(AppError::NotFound(format!("Role name {}", role)))?;

        let result = sqlx::query!(
            r#"
            INSERT INTO public.user_roles (user_id, role_id)
            VALUES ($1, $2)
            ON CONFLICT (user_id, role_id) DO NOTHING;
            "#,
            user_id,
            role_id
        )
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}