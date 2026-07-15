use async_trait::async_trait;
use sqlx::PgPool;

use crate::{errors::{AppError, AppResult}, models::user_roles::UserRoles};

#[async_trait]
pub trait UserRoleRepositoryTrait: Send + Sync {
    async fn check_assign_role(&self, user_id: i64, role_id: i64) -> AppResult<bool>;
    async fn assign_role_to_user(&self, user_id: i64, role_id: i64) -> AppResult<UserRoles>;
}

pub struct UserRoleRepository {
    db: PgPool
}

impl UserRoleRepository {
    pub fn new(db: PgPool) -> Self {
        Self {
            db
        }
    }
}

#[async_trait]
impl UserRoleRepositoryTrait for UserRoleRepository {
    async fn assign_role_to_user(&self, user_id: i64, role_id: i64) -> AppResult<UserRoles>{
        let user_role = sqlx::query_as!(
            UserRoles,
            r#"
            INSERT INTO public.user_roles (user_id, role_id)
            VALUES ($1, $2)
            RETURNING id, user_id, role_id, created_at, updated_at
            "#,
            user_id,
            role_id
        )
        .fetch_one(&self.db)
        .await?;

        Ok(user_role)
    }
}