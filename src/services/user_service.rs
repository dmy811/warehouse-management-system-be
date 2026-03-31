use std::sync::Arc;

use async_trait::async_trait;

use crate::{dtos::{UserResponse, user_dto::CreateUserRequest}, errors::{AppError, AppResult}, repositories::user_repository::UserRepositoryTrait, utils::crypto::hash_password};

#[async_trait]
pub trait UserServiceTrait: Send + Sync {
    async fn create(&self, req: CreateUserRequest) -> AppResult<UserResponse>;
}

pub struct UserService<R: UserRepositoryTrait> {
    repo: Arc<R>
}

impl<R: UserRepositoryTrait> UserService<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self {
            repo
        }
    }
}

#[async_trait]
impl<R: UserRepositoryTrait> UserServiceTrait for UserService<R>  {
    async fn create(&self, req: CreateUserRequest) -> AppResult<UserResponse> {
        if self.repo.email_exists(&req.email).await? {
            return Err(AppError::Conflict(format!(
                "Email '{}' is already registered",
                req.email
            )))
        }

        let password_hash = tokio::task::spawn_blocking({
            let password = req.password.clone();
            move || hash_password(&password)
        })
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Thread join error: {}", e)))??;

        let user = self
            .repo
            .create(&req.name, &req.email, &password_hash, req.phone.as_deref(), role)
    }
}