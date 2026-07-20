use std::sync::Arc;
use async_trait::async_trait;

use crate::{errors::{AppError, AppResult}, models::user_roles::UserRoles, repositories::{role_repository::RoleRepositoryTrait, user_repository::UserRepositoryTrait, user_role_repository::UserRoleRepositoryTrait}};

#[async_trait]
pub trait UserRoleServiceTrait: Send + Sync {
    async fn assign_role_to_user(&self, user_id: i64, role_id: i64) -> AppResult<UserRoles>;
}

pub struct UserRoleService<UR: UserRoleRepositoryTrait, U: UserRepositoryTrait, R: RoleRepositoryTrait> {
    pub repo: Arc<UR>,
    pub user_repo: Arc<U>,
    pub role_repo: Arc<R>
}

impl <UR: UserRoleRepositoryTrait, U: UserRepositoryTrait, R: RoleRepositoryTrait> UserRoleService<UR, U, R> {
    pub fn new(repo: Arc<UR>, user_repo: Arc<U>, role_repo: Arc<R>) -> Self {
        Self {
            repo,
            user_repo,
            role_repo
        }
    }
}

#[async_trait]
impl <UR: UserRoleRepositoryTrait, U: UserRepositoryTrait, R: RoleRepositoryTrait> UserRoleServiceTrait for UserRoleService<UR, U, R> {
    async fn assign_role_to_user(&self, user_id: i64, role_id: i64) -> AppResult<UserRoles> {
        self.user_repo
        .find_user_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User with id {}", user_id)))?;

        self.role_repo
        .find_role_by_id(role_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Role with id {}", role_id)))?;

        if self.repo.check_assign_role(user_id, role_id).await? {
            return Err(AppError::NotFound(format!("Role with id {} is already assigned to user with id {}", role_id, user_id)))
        }

        let user_role = self.repo.assign_role_to_user(user_id, role_id).await?;

        Ok(user_role)
    }
}