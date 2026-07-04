use std::sync::Arc;

use async_trait::async_trait;

use crate::{dtos::rack_dto::{CreateRackRequest, RackResponse, RackSummary, UpdateRackRequest}, errors::{AppError, AppResult}, models::params::rack_params::{CreateRackParams, UpdateRackParams}, repositories::{WarehouseRepositoryTrait, rack_repository::RackRepositoryTrait}, response::{ListQuery, PaginatedResponse}};

#[async_trait]
pub trait RackServiceTrait: Send + Sync {
    async fn get_all_racks(&self, warehouse_id: i64, query: &ListQuery) -> AppResult<PaginatedResponse<RackSummary>>;
    async fn get_rack_by_id(&self, id: i64, warehouse_id: i64) -> AppResult<RackSummary>;
    async fn create_rack(&self, req: CreateRackRequest) -> AppResult<RackResponse>;
    async fn update_rack(&self, id: i64, req: UpdateRackRequest) -> AppResult<RackResponse>;
    async fn soft_delete(&self, id: i64, warehouse_id: i64) -> AppResult<()>;
}

pub struct RackService<R: RackRepositoryTrait, W: WarehouseRepositoryTrait> {
    pub repo: Arc<R>,
    pub warehouse_repo: Arc<W>
}

impl <R: RackRepositoryTrait, W: WarehouseRepositoryTrait> RackService<R, W> {
    pub fn new(repo: Arc<R>, warehouse_repo: Arc<W>) -> Self {
        Self {
            repo, 
            warehouse_repo
        }
    }
}

#[async_trait]
impl <R: RackRepositoryTrait, W: WarehouseRepositoryTrait> RackServiceTrait for RackService<R, W> {
    async fn get_all_racks(&self, warehouse_id: i64, query: &ListQuery) -> AppResult<PaginatedResponse<RackSummary>> {
        let (racks, total) = self
            .repo
            .find_all(warehouse_id, &query)
            .await?;

        let items: Vec<RackSummary> = racks
            .into_iter()
            .map(RackSummary::from)
            .collect();

        Ok(PaginatedResponse::new(items, total, query.page, query.per_page))
    }
    async fn get_rack_by_id(&self, id: i64, warehouse_id: i64) -> AppResult<RackSummary> {
        let rack = self
            .repo
            .find_by_id(id, warehouse_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Rack with id {}", id)))?;

        Ok(RackSummary::from(rack))
    }
    async fn create_rack(&self, req: CreateRackRequest) -> AppResult<RackResponse> {
        self
            .warehouse_repo
            .find_warehouse_by_id(req.warehouse_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Warehouse with id {}", req.warehouse_id)))?;

        if self.repo.code_exists(&req.code, req.warehouse_id, None).await? {
            return Err(AppError::Conflict(format!(
                "Rack with code '{}'",
                req.code
            )));
        }

        let rack_repo_request = CreateRackParams {
            warehouse_id: req.warehouse_id,
            code: &req.code,
            zone: req.zone.as_deref(),
            level: req.level,
            capacity: req.capacity,
            description: req.description.as_deref()
        };

        let rack = self
            .repo
            .create(rack_repo_request)
            .await?;
        Ok(RackResponse::from(rack))
    }
    async fn update_rack(&self, id: i64, req: UpdateRackRequest) -> AppResult<RackResponse> {
        // we need to check if the user is capable to access the warehouse, it can be solve with checking the user warehouse table
        self
            .warehouse_repo
            .find_warehouse_by_id(req.warehouse_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Warehouse with id {}", req.warehouse_id)))?;

        self
            .repo
            .find_by_id(id, req.warehouse_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Rack with id {}", id)))?;

        if let Some(ref code) = req.code {
            if self.repo.code_exists(code, req.warehouse_id, Some(id)).await? {
                return Err(AppError::Conflict(format!(
                    "Rack with code '{}'",
                    code
                )));
            }
        }

        let update_rack_request = UpdateRackParams {
            code: req.code.as_deref(),
            zone: req.zone.as_deref(),
            level: req.level,
            capacity: req.capacity,
            description: req.description.as_deref()
        };

        let rack = self
            .repo
            .update(id, req.warehouse_id, update_rack_request)
            .await?
            .ok_or_else(|| AppError::InternalUi("Failed to update rack".to_string()))?;

        Ok(RackResponse::from(rack))
    }
    async fn soft_delete(&self, id: i64, warehouse_id: i64) -> AppResult<()> {
        self
            .repo
            .find_by_id(id, req.warehouse_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Rack with id {}", id)))?;
    }
}