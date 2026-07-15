use async_trait::async_trait;
use sqlx::PgPool;

use crate::{errors::AppResult, models::Role};

#[async_trait]
pub trait RoleRepositoryTrait: Send + Sync {
    async fn find_all_roles(&self) -> AppResult<Vec<Role>>;
    async fn find_role_by_id(&self, id: i64) -> AppResult<Option<Role>>;
    async fn check_name_exists(&self, name: &str, exclude_id: Option<i64>) -> AppResult<bool>;
    async fn create_roles(&self, name: &str) -> AppResult<Role>;
    async fn update_roles(&self, id: i64, name: Option<&str>) -> AppResult<Option<Role>>;
    async fn delete_roles(&self, id: i64) -> AppResult<()>;
}

pub struct RoleRepository {
    db: PgPool
}

impl RoleRepository {
    pub fn new(db: PgPool) -> Self {
        Self {
            db
        }
    }
}

#[async_trait]
impl RoleRepositoryTrait for RoleRepository {
    async fn find_all_roles(&self) -> AppResult<Vec<Role>> {
        let roles = sqlx::query_as!(
            Role,
            r#"
            SELECT id, name, created_at
            FROM roles
            "#
        )
        .fetch_all(&self.db)
        .await?;

        Ok(roles)
    }

    async fn find_role_by_id(&self, id: i64) -> AppResult<Option<Role>> {
        let role = sqlx::query_as!(
            Role,
            r#"
            SELECT id, name, created_at
            FROM roles WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(role)
    }

    async fn check_name_exists(&self, name: &str, exclude_id: Option<i64>) -> AppResult<bool> {
        let exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM roles
                WHERE LOWER(name) = LOWER($1)
                AND ($2::BIGINT IS NULL OR id != $2)
            )
            "#,
            name,
            exclude_id
        )
        .fetch_one(&self.db)
        .await?
        .unwrap_or(false);

        Ok(exists)
    }

    async fn create_roles(&self, name: &str) -> AppResult<Role> {
        let role = sqlx::query_as!(
            Role,
            r#"
            INSERT INTO roles (name)
            VALUES ($1)
            RETURNING id, name, created_at
            "#,
            name
        )
        .fetch_one(&self.db)
        .await?;

        Ok(role)
    }
    
    async fn update_roles(&self, id: i64, name: Option<&str>) -> AppResult<Option<Role>> {
        let role = sqlx::query_as!(
            Role,
            r#"
            UPDATE roles SET
                name = COALESCE($2, name)
            WHERE id = $1
            RETURNING id, name, created_at
            "#,
            id,
            name
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(role)
    }

    async fn delete_roles(&self, id: i64) -> AppResult<()> {
        sqlx::query!(
            r#"
            DELETE FROM roles
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
}