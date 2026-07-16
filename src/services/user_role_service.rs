use std::sync::Arc;
use async_trait::async_trait;

use crate::{errors::AppResult, models::user_roles::UserRoles, repositories::user_role_repository::UserRoleRepositoryTrait};

#[async_trait]
pub trait UserRoleServiceTrait: Send + Sync {
    async fn assign_role_to_user(&self, user_id: i64, role_id: i64) -> AppResult<UserRoles>;
}

pub struct UserRoleService<R: UserRoleRepositoryTrait> {
    pub repo: Arc<R>
}

impl <R: UserRoleRepositoryTrait> UserRoleService<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self {
            repo
        }
    }
}

#[async_trait]
impl <R: UserRoleRepositoryTrait> UserRoleServiceTrait for UserRoleService<R> {
    async fn assign_role_to_user(&self, user_id: i64, role_id: i64) -> AppResult<UserRoles> {
        
    }
}