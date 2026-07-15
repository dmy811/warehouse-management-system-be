use std::sync::Arc;
use async_trait::async_trait;

use crate::{errors::{AppError, AppResult}, models::Role, repositories::roles_repository::RoleRepositoryTrait};

#[async_trait]
pub trait RoleServiceTrait: Send + Sync {
    async fn get_all_roles(&self) -> AppResult<Vec<Role>>;
    async fn get_role_by_id(&self, id: i64) -> AppResult<Role>;
    async fn create_roles(&self, name: &str) -> AppResult<Role>;
    async fn update_roles(&self, id: i64, name: Option<&str>) -> AppResult<Role>;
    async fn delete_roles(&self, id: i64) -> AppResult<()>;
}

pub struct RoleService<R: RoleRepositoryTrait> {
    pub repo: Arc<R>
}

impl <R: RoleRepositoryTrait> RoleService<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self {
            repo
        }
    }
}

#[async_trait]
impl <R: RoleRepositoryTrait> RoleServiceTrait for RoleService<R> {
    async fn get_all_roles(&self) -> AppResult<Vec<Role>> {
        let roles = self.repo.find_all_roles().await?;
        Ok(roles)
    }

    async fn get_role_by_id(&self, id: i64) -> AppResult<Role> {
        let role = self
            .repo
            .find_role_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Roles with id {}", id)))?;

        Ok(role)
    }

    async fn create_roles(&self, name: &str) -> AppResult<Role> {
        let role = self
            .repo
            .create_roles(name)
            .await?;

        Ok(role)
    }

    async fn update_roles(&self, id: i64, name: Option<&str>) -> AppResult<Role> {
        self
            .repo
            .find_role_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Roles with id {}", id)))?;

        let role = self
            .repo
            .update_roles(id, name)
            .await?
            .ok_or_else(|| AppError::InternalUi("Failed to update role".to_string()))?;

        Ok(role)
    }

    async fn delete_roles(&self, id: i64) -> AppResult<()> {
        self
            .repo
            .find_role_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Roles with id {}", id)))?;

        self.repo.delete_roles(id).await?;

        Ok(())
    }
}
