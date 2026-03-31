use std::sync::Arc;

use async_trait::async_trait;
use tracing::info;

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
            .create(&req.name, &req.email, &password_hash, req.phone.as_deref(), &req.role)
            .await?;

        let user_with_role = self
            .repo
            .find_by_id(user.id)
            .await?
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("User not found after insert")))?;

        info!(user_id = user.id, email = %req.email, "New user created");
        
        Ok(UserResponse::from(user_with_role))
    }
}