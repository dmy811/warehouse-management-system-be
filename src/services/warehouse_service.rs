use std::sync::Arc;

use async_trait::async_trait;
use tracing::info;

use crate::{dtos::{CreateWarehouseRequest, UpdateWarehouseRequest, WarehouseResponse, WarehouseSummary}, errors::{AppError, AppResult}, repositories::WarehouseRepositoryTrait, response::{PaginatedResponse, ListQuery}};

#[async_trait]
pub trait WarehouseServiceTrait: Send + Sync {
    async fn list_all_warehouses(&self, query: ListQuery) -> AppResult<PaginatedResponse<WarehouseSummary>>;
    async fn get_warehouse_by_id(&self, id: i64) -> AppResult<WarehouseResponse>;
    async fn create_warehouse(&self, req: CreateWarehouseRequest, actor_id: i64) -> AppResult<WarehouseResponse>;
    async fn update_warehouse(&self, id: i64, req: UpdateWarehouseRequest, actor_id: i64) -> AppResult<WarehouseResponse>;
    async fn delete_warehouse(&self, id: i64, actor_id: i64) -> AppResult<()>;
    async fn update_warehouse_photo(&self, id: i64, photo_url: &str, actor_id: i64) -> AppResult<()>;
    async fn delete_warehouse_photo(&self, id: i64, actor_id: i64) -> AppResult<()>;
}

pub struct WarehouseService<R: WarehouseRepositoryTrait> {
    repo: Arc<R>
}

impl<R: WarehouseRepositoryTrait> WarehouseService<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self {
            repo
        }
    }
}

#[async_trait]
impl<R: WarehouseRepositoryTrait> WarehouseServiceTrait for WarehouseService<R> {
    async fn list_all_warehouses(&self, query: ListQuery) -> AppResult<PaginatedResponse<WarehouseSummary>> {
        let (warehouses, total) = self.repo.find_all_warehouses(&query).await?;

        let items: Vec<WarehouseSummary> = warehouses
            .into_iter() // into_iter() means take ownership from every element in collection (Vec), if it use iter() means borrow (&T), if it use iter_mut() means borrow mutable (&mut T)
            .map(WarehouseSummary::from) // it same as like .map(|w| WarehouseSummary::from(w))
            .collect(); // change iterator into Vec

        Ok(PaginatedResponse::new(items, total, query.page, query.per_page))
    }

    async fn get_warehouse_by_id(&self, id: i64) -> AppResult<WarehouseResponse> {
        let warehouse = self
            .repo
            .find_warehouse_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Warehouse with id {}", id)))?;

        Ok(WarehouseResponse::from(warehouse))
    }

    async fn create_warehouse(&self, req: CreateWarehouseRequest, actor_id: i64) -> AppResult<WarehouseResponse> {
        if self.repo.check_name_exists(&req.name, None).await? {
            return Err(AppError::Conflict(format!(
                "Warehouse with name '{}' already exists",
                req.name
            )));
        }

        let warehouse = self
            .repo
            .create_warehouse(&req.name, &req.address, req.phone.as_deref(), req.photo.as_deref())
            .await?;

        info!(
            warehouse_id = warehouse.id,
            warehouse_name = %warehouse.name,
            actor_id = actor_id,
            "Warehouse created"
        );

        Ok(WarehouseResponse::from(warehouse))
    }

    async fn update_warehouse(&self, id: i64, req: UpdateWarehouseRequest, actor_id: i64) -> AppResult<WarehouseResponse> {
        let phone = req.phone.as_deref().and_then(|v| {
            let v = v.trim();
            if v.is_empty() {
                None
            } else {
                Some(v)
            }
        });
        let photo = req.photo.as_deref().and_then(|v| {
            let v = v.trim();
            if v.is_empty() {
                None
            } else {
                Some(v)
            }
        });

        self.repo
            .find_warehouse_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Warehouse with id {}", id)))?;

        if let Some(ref name) = req.name {
            if self.repo.check_name_exists(name, Some(id)).await? {
                return Err(AppError::Conflict(format!(
                    "Warehouse with name '{}' already exists",
                    name
                )));
            }
        }

        let warehouse = self
            .repo
            .update_warehouse(
                id,
                req.name.as_deref(),
                req.address.as_deref(),
                phone,
                photo
            )
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Warehouse with id {}", id)))?;

        info!(
            warehouse_id = id,
            actor_id = actor_id,
            "Warehouse updated"
        );
 
        Ok(WarehouseResponse::from(warehouse))
    }
    async fn delete_warehouse(&self, id: i64, actor_id: i64) -> AppResult<()> {
        self.repo
            .find_warehouse_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Warehouse with id {}", id)))?;

        self.repo.warehouse_soft_delete(id).await?;

        info!(
            warehouse_id = id,
            actor_id = actor_id,
            "Warehouse soft-deleted"
        );
 
        Ok(())

    }
    async fn update_warehouse_photo(&self, id: i64, photo_url: &str, actor_id: i64) -> AppResult<()> {
        self.repo
            .find_warehouse_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Warehouse with id {}", id)))?;
        
        self.repo.update_warehouse_photo(id, photo_url).await?;
 
        info!(
            warehouse_id = id,
            actor_id = actor_id,
            photo_url = photo_url,
            "Warehouse photo updated"
        );
 
        Ok(())
    }
    async fn delete_warehouse_photo(&self, id: i64, actor_id: i64) -> AppResult<()> {
        self.repo
            .find_warehouse_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Warehouse with id {}", id)))?;

        self.repo.clear_warehouse_photo(id).await?;
 
        info!(
            warehouse_id = id,
            actor_id = actor_id,
            "Warehouse photo deleted"
        );
 
        Ok(())
    }
}