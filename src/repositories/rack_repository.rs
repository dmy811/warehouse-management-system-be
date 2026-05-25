use async_trait::async_trait;
use sqlx::PgPool;
use crate::{errors::AppResult, models::{Rack, RackWithStats}, response::ListQuery};

#[async_trait]
pub trait RackRepositoryTrait: Send + Sync {
    async fn find_all_racks(&self, query: &ListQuery) -> AppResult<(Vec<RackWithStats>, i64)>;
    async fn find_rack_by_id(&self, id: i64) -> AppResult<Option<Rack>>;
    async fn check_code_exists(&self, code: &str) -> AppResult<bool>;
    async fn create_rack(&self, )
}
