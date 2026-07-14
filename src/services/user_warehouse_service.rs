use async_trait::async_trait;

use crate::{errors::AppResult, repositories::user_warehouse_repository::UserWarehouseRepositoryTrait};

#[async_trait]
pub trait UserWarehouseServiceTrait: Send + Sync {
    async fn assign_warehouse_to_user(&self, user_id: i64, warehouse_id: i64) -> AppResult<()>;
}

pub struct UserWarehouseService <R: UserWarehouseRepositoryTrait> {
    pub repo: R
}

impl <R: UserWarehouseRepositoryTrait> UserWarehouseService<R> {
    pub fn new(repo: R) -> Self {
        Self {
            repo
        }
    }
}

#[async_trait]
impl <R: UserWarehouseRepositoryTrait> UserWarehouseServiceTrait for UserWarehouseService<R> {
    async fn assign_warehouse_to_user
}