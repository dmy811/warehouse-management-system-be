use async_trait::async_trait;

use crate::{dtos::rack_dto::RackSummary, errors::AppResult, response::{ListQuery, PaginatedResponse}};

#[async_trait]
pub trait RackServiceTrait: Send + Sync {
    async fn get_all_racks(&self, warehouse_id: i64, query: &ListQuery) -> AppResult<PaginatedResponse<RackSummary>>;
    async fn get_rack_by_id(&self, id: i64, warehouse_id: i64) -> AppResult<Option<RackSummary>>;
    
}