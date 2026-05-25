use std::sync::Arc;

use async_trait::async_trait;
use tracing::info;

use crate::{dtos::{UserResponse, user_dto::{AddRoleRequest, CreateUserRequest, UpdateUserRequest}}, errors::{AppError, AppResult}, repositories::user_repository::UserRepositoryTrait, response::{ListQuery, PaginatedResponse}, utils::crypto::hash_password};

#[async_trait]
pub trait UserServiceTrait: Send + Sync {
    async fn create_user(&self, req: CreateUserRequest) -> AppResult<UserResponse>;
    async fn list_all_users(&self, query: ListQuery) -> AppResult<PaginatedResponse<UserResponse>>;
    async fn find_user_by_id(&self, user_id: i64) -> AppResult<UserResponse>;
    async fn update_user(&self, user_id: i64, req: UpdateUserRequest) -> AppResult<UserResponse>;
    async fn user_soft_delete(&self, user_id: i64) -> AppResult<()>;
    async fn user_hard_delete(&self, user_id: i64) -> AppResult<()>;
    async fn add_role(&self, req: AddRoleRequest) -> AppResult<()>;
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
    async fn create_user(&self, req: CreateUserRequest) -> AppResult<UserResponse> {
        if self.repo.check_email_exists(&req.email).await? {
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
            .create_user(&req.name, &req.email, &password_hash, req.phone.as_deref(), &req.role)
            .await?;

        let user_with_role = self
            .repo
            .find_user_by_id(user.id)
            .await?
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("User not found after insert")))?;

        info!(user_id = user.id, email = %req.email, "New user created");
        
        Ok(UserResponse::from(user_with_role))
    }

    async fn list_all_users(&self, query: ListQuery) -> AppResult<PaginatedResponse<UserResponse>> {
        let (users, total) = self.repo.find_all_users(&query).await?;

        let items: Vec<UserResponse> = users
            .into_iter()
            .map(UserResponse::from)
            .collect();

        Ok(PaginatedResponse::new(items, total, query.page, query.per_page))

    }

    async fn find_user_by_id(&self, user_id: i64) -> AppResult<UserResponse> {
        let user = self.repo
            .find_user_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User with id {}", user_id)))?;

        Ok(UserResponse::from(user))
    }

    async fn update_user(&self, user_id: i64, req: UpdateUserRequest) -> AppResult<UserResponse> {
        let user = self.repo
            .find_user_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User with id {}", user_id)))?;

        if let Some(email) = &req.email {
            if self.repo.check_email_exists(email).await? {
                return Err(AppError::Conflict(format!(
                    "Email '{}' is already registered",
                    email
                )));
            }
        }
        self.repo
            .update_user(user_id, req.name.as_deref(), req.email.as_deref(), req.phone.as_deref())
            .await?;

        Ok(UserResponse::from(user))
    }

    async fn user_soft_delete(&self, user_id: i64) -> AppResult<()> {
        self.repo
            .find_user_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User with id {}", user_id)))?;

        self.repo.user_soft_delete(user_id).await?;


        info!(user_id = user_id, "User soft deleted");

        Ok(())
    }
    async fn user_hard_delete(&self, user_id: i64) -> AppResult<()> {
        self.repo
            .find_user_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User with id {}", user_id)))?;

        self.repo.user_hard_delete(user_id).await?;


        info!(user_id = user_id, "User hard deleted");

        Ok(())
    }

    async fn add_role(&self, req: AddRoleRequest) -> AppResult<()>{
        self.repo
            .find_user_by_id(req.user_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User with id {}", user_id)))?;

        self.repo
            .add_role(req.user_id, &req.role)
            .await?;

        Ok(())
    }
}