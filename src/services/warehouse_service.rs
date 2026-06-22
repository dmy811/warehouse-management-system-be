use std::sync::Arc;

use async_trait::async_trait;
use tracing::info;

use crate::{dtos::{CreateWarehouseRequest, UpdateWarehouseRequest, WarehouseResponse, WarehouseSummary}, errors::{AppError, AppResult}, repositories::WarehouseRepositoryTrait, response::{PaginatedResponse, ListQuery}};

#[async_trait]
pub trait WarehouseServiceTrait: Send + Sync {
    async fn get_all_warehouses(&self, query: ListQuery) -> AppResult<PaginatedResponse<WarehouseSummary>>;
    async fn get_warehouse_by_id(&self, id: i64) -> AppResult<WarehouseSummary>;
    async fn create_warehouse(&self, req: CreateWarehouseRequest, actor_id: i64) -> AppResult<WarehouseResponse>;
    async fn update_warehouse(&self, warehouse_id: i64, req: UpdateWarehouseRequest, actor_id: i64) -> AppResult<WarehouseResponse>;
    async fn delete_warehouse_soft(&self, warehouse_id: i64, actor_id: i64) -> AppResult<()>;
    async fn delete_warehouse_hard(&self, warehouse_id: i64, actor_id: i64) -> AppResult<()>;
    async fn update_warehouse_photo(&self, warehouse_id: i64, photo_url: &str, actor_id: i64) -> AppResult<()>;
    async fn delete_warehouse_photo(&self, warehouse_id: i64, actor_id: i64) -> AppResult<()>;
    async fn assign_warehouse_to_user(&self, user_id: i64, warehouse_id: i64) -> AppResult<()>;
}

pub struct WarehouseService<R: WarehouseRepositoryTrait> {
    repo: Arc<R>
}

impl<R: WarehouseRepositoryTrait> WarehouseService<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl<R: WarehouseRepositoryTrait> WarehouseServiceTrait for WarehouseService<R> {
    async fn get_all_warehouses(&self, query: ListQuery) -> AppResult<PaginatedResponse<WarehouseSummary>> {
        let (warehouses, total) = self.repo.find_all_warehouses(&query).await?;

        let items: Vec<WarehouseSummary> = warehouses
            .into_iter()
            .map(WarehouseSummary::from)
            .collect();

        Ok(PaginatedResponse::new(items, total, query.page, query.per_page))
    }

    async fn get_warehouse_by_id(&self, warehouse_id: i64) -> AppResult<WarehouseSummary> {
        let warehouse = self
            .repo
            .find_warehouse_by_id(warehouse_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Warehouse with id {}", warehouse_id)))?;

        Ok(WarehouseSummary::from(warehouse))
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

    async fn update_warehouse(&self, warehouse_id: i64, req: UpdateWarehouseRequest, actor_id: i64) -> AppResult<WarehouseResponse> {
        let phone = req.phone.as_deref().and_then(|v| {
            let v = v.trim();
            if v.is_empty() { None } else { Some(v) }
        });
        let photo = req.photo.as_deref().and_then(|v| {
            let v = v.trim();
            if v.is_empty() { None } else { Some(v) }
        });

        // Validasi keberadaan awal
        self.repo
            .find_warehouse_by_id(warehouse_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Warehouse with id {}", warehouse_id)))?;

        if let Some(ref name) = req.name {
            if self.repo.check_name_exists(name, Some(warehouse_id)).await? {
                return Err(AppError::Conflict(format!(
                    "Warehouse with name '{}' already exists",
                    name
                )));
            }
        }

        let warehouse = self
            .repo
            .update_warehouse(
                warehouse_id,
                req.name.as_deref(),
                req.address.as_deref(),
                phone,
                photo
            )
            .await?
            // Jika di atas sudah lolos find_by_id, maka None di sini murni karena isu konkurensi data atau kegagalan internal database
            .ok_or_else(|| AppError::InternalUi("Failed to update warehouse".to_string()))?;

        info!(
            warehouse_id = warehouse_id,
            actor_id = actor_id,
            "Warehouse updated"
        );
 
        Ok(WarehouseResponse::from(warehouse))
    }

    async fn delete_warehouse_soft(&self, warehouse_id: i64, actor_id: i64) -> AppResult<()> {
        self.repo
            .find_warehouse_by_id(warehouse_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Warehouse with id {}", warehouse_id)))?;

        self.repo.warehouse_soft_delete(warehouse_id).await?;

        info!(warehouse_id, actor_id, "Warehouse soft-deleted");
        Ok(())
    }

    async fn delete_warehouse_hard(&self, warehouse_id: i64, actor_id: i64) -> AppResult<()> {
        self.repo
            .find_warehouse_by_id(warehouse_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Warehouse with id {}", warehouse_id)))?;

        self.repo.warehouse_hard_delete(warehouse_id).await?;

        info!(warehouse_id, actor_id, "Warehouse hard-deleted");
        Ok(())
    }

    async fn update_warehouse_photo(&self, warehouse_id: i64, photo_url: &str, actor_id: i64) -> AppResult<()> {
        self.repo
            .find_warehouse_by_id(warehouse_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Warehouse with id {}", warehouse_id)))?;
        
        self.repo.update_warehouse_photo(warehouse_id, photo_url).await?;
 
        info!(warehouse_id, actor_id, photo_url, "Warehouse photo updated");
        Ok(())
    }

    async fn delete_warehouse_photo(&self, warehouse_id: i64, actor_id: i64) -> AppResult<()> {
        self.repo
            .find_warehouse_by_id(warehouse_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Warehouse with id {}", warehouse_id)))?;

        self.repo.clear_warehouse_photo(warehouse_id).await?;
 
        info!(warehouse_id, actor_id, "Warehouse photo deleted");
        Ok(())
    }

    async fn assign_warehouse_to_user(&self, user_id: i64, warehouse_id: i64) -> AppResult<()> {
        if !self.repo.check_user_existing(user_id).await? {
            return Err(AppError::NotFound(format!("User with id {}", user_id)));
        }

        self.repo
            .find_warehouse_by_id(warehouse_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Warehouse with id {}", warehouse_id)))?;

        if self.repo.check_existing_warehouse_in_user(user_id, warehouse_id).await? {
            return Err(AppError::Conflict(format!(
                "Warehouse with id {} is already assigned to user with id {}", 
                warehouse_id, user_id
            )));
        }

        self.repo.assign_warehouse_to_user(user_id, warehouse_id).await?;

        info!(user_id, warehouse_id, "Warehouse successfully assigned to user");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use chrono::Utc;
use mockall::predicate::*;

    use crate::{
        dtos::{CreateWarehouseRequest, UpdateWarehouseRequest}, errors::AppError, models::{Warehouse, WarehouseWithStats}, repositories::warehouse_repository::MockWarehouseRepositoryTrait
    };

    fn setup_service(repo: MockWarehouseRepositoryTrait) -> WarehouseService<MockWarehouseRepositoryTrait> {
        WarehouseService::new(Arc::new(repo))
    }

    fn mock_warehouse(id: i64, name: &str, address: &str) -> Warehouse {
        Warehouse {
            id,
            name: name.to_string(),
            address: address.to_string(),
            photo: None,
            phone: None,
            deleted_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn mock_warehouse_with_stats(id: i64, name: &str, address: &str) -> WarehouseWithStats {
        WarehouseWithStats {
            id,
            name: name.to_string(),
            address: address.to_string(),
            photo: None,
            phone: None,
            deleted_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            total_products: Some(10),
            total_racks: Some(5)
        }
    }

    #[tokio::test]
    async fn test_get_warehouse_by_id_success(){
        let mut mock_repo = MockWarehouseRepositoryTrait::new();
        mock_repo
            .expect_find_warehouse_by_id()
            .with(eq(1))
            .times(1)
            .returning(|_| Ok(Some(mock_warehouse_with_stats(1, "Gudang Jakarta", "Jl. Merdeka No 1"))));

        let service = setup_service(mock_repo);
        let result = service.get_warehouse_by_id(1).await;

        
        assert!(result.is_ok(), "Expected to return Ok(WarehouseSummary)");
        let warehouse_summary = result.unwrap(); 

        assert_eq!(warehouse_summary.id, 1); 
        assert_eq!(warehouse_summary.name, "Gudang Jakarta");
    }

    #[tokio::test]
    async fn test_get_warehouse_by_id_not_found(){
        let mut mock_repo = MockWarehouseRepositoryTrait::new();
        mock_repo.expect_find_warehouse_by_id()
        .with(eq(99))
        .times(1)
        .returning(|_| Ok(None));

        let service = setup_service(mock_repo);
        let result = service.get_warehouse_by_id(99).await;

        assert!(result.is_err(), "Exptected to return an Error Not Found");

        let err = result.unwrap_err();

        assert!(matches!(err, AppError::NotFound(_)), "The expected type is NotFound");
        assert_eq!(err.to_string(), "Warehouse with id 99 not found");
    }

    #[tokio::test]
    async fn test_create_warehouse_success() {
        let req = CreateWarehouseRequest {
            name: "Gudang Baru".to_string(),
            address: "Jl. Baru".to_string(),
            phone: Some("088556637748".to_string()),
            photo: None
        };

        let mut mock_repo = MockWarehouseRepositoryTrait::new();
        
        mock_repo
            .expect_check_name_exists()
            .with(eq("Gudang Baru".to_string()), eq(None))
            .times(1)
            .returning(|_, _| Ok(false));

        mock_repo
            .expect_create_warehouse()
            .withf(|name: &str, address: &str, phone: &Option<&str>, photo: &Option<&str>| {
            name == "Gudang Baru" 
                && address == "Jl. Baru" 
                && *phone == Some("088556637748") 
                && photo.is_none()
        })
            .times(1)
            .returning(|_, _, _, _| Ok(mock_warehouse(1, "Gudang Baru", "Jl. Baru")));

        let service = setup_service(mock_repo);
        let result = service.create_warehouse(req, 1).await;

        assert!(result.is_ok(), "Expected to return Ok(WarehouseResponse)");
        let response = result.unwrap();
        assert_eq!(response.id, 1);
        assert_eq!(response.name, "Gudang Baru");
    }

    #[tokio::test]
    async fn test_create_warehouse_conflict(){
        let req = CreateWarehouseRequest {
            name: "Gudang Duplikat".to_string(),
            address: "Jl. Duplikat".to_string(),
            phone: None,
            photo: None,
        };

        let mut mock_repo = MockWarehouseRepositoryTrait::new();
        
        mock_repo
            .expect_check_name_exists()
            .with(eq("Gudang Duplikat".to_string()), eq(None))
            .times(1)
            .returning(|_, _| Ok(true));

        let service = setup_service(mock_repo);
        let result = service.create_warehouse(req, 1).await;

        assert!(result.is_err(), "Expected to return an Error Conflict");

        let err = result.unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)), "The expected type is Conflict");
        assert_eq!(err.to_string(), "Warehouse with name 'Gudang Duplikat' already exists");
    }

    #[tokio::test]
    async fn test_assign_warehouse_success() {
        let mut mock_repo = MockWarehouseRepositoryTrait::new();
        mock_repo
            .expect_check_user_existing()
            .with(eq(10))
            .times(1)
            .returning(|_| Ok(true));

        mock_repo
            .expect_find_warehouse_by_id()
            .with(eq(20))
            .times(1)
            .returning(|_| Ok(Some(mock_warehouse_with_stats(20, "warehouse", "Jl jambu"))));

        mock_repo
            .expect_check_existing_warehouse_in_user()
            .with(eq(10), eq(20))
            .times(1)
            .returning(|_, _| Ok(false));

        mock_repo
            .expect_assign_warehouse_to_user()
            .with(eq(10), eq(20))
            .times(1)
            .returning(|_,_| Ok(()));

        let service = setup_service(mock_repo);
        let result = service.assign_warehouse_to_user(10, 20).await;

        assert!(result.is_ok(), "Expected to successfull assigned warehouse to user");

    }
    
    #[tokio::test]
    async fn test_assign_warehouse_user_not_exists() {
        let mut mock_repo = MockWarehouseRepositoryTrait::new();
        mock_repo
            .expect_check_user_existing()
            .with(eq(10))
            .times(1)
            .returning(|_| Ok(false));

        let service = setup_service(mock_repo);
        let result = service.assign_warehouse_to_user(10, 20).await;

        assert!(result.is_err(), "Expected to error user not exists");
        let err = result.unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)), "The expected type is NotFound");
        assert_eq!(err.to_string(), "User with id 10 not found");

    }

    #[tokio::test]
    async fn test_assign_warehouse_warehouse_not_exists() {
        let mut mock_repo = MockWarehouseRepositoryTrait::new();
        mock_repo
            .expect_check_user_existing()
            .with(eq(10))
            .times(1)
            .returning(|_| Ok(true));

        mock_repo
            .expect_find_warehouse_by_id()
            .with(eq(20))
            .times(1)
            .returning(|_| Ok(None));

        let service = setup_service(mock_repo);
        let result = service.assign_warehouse_to_user(10, 20).await;

        assert!(result.is_err(), "Expected to error warehouse not exists");
        let err = result.unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)), "The expected type is NotFound");
        assert_eq!(err.to_string(), "Warehouse with id 20 not found");

    }

    #[tokio::test]
    async fn test_assign_warehouse_warehouse_already_assigned_to_user() {
        let mut mock_repo = MockWarehouseRepositoryTrait::new();
        mock_repo
            .expect_check_user_existing()
            .with(eq(10))
            .times(1)
            .returning(|_| Ok(true));

        mock_repo
            .expect_find_warehouse_by_id()
            .with(eq(20))
            .times(1)
            .returning(|_| Ok(Some(mock_warehouse_with_stats(20, "warehouse", "Jl jambu"))));

        mock_repo
            .expect_check_existing_warehouse_in_user()
            .with(eq(10), eq(20))
            .times(1)
            .returning(|_, _| Ok(true));

        let service = setup_service(mock_repo);
        let result = service.assign_warehouse_to_user(10, 20).await;

        assert!(result.is_err(), "Expected to error warehouse not exists");
        let err = result.unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)), "The expected type is Conflict");
        assert_eq!(err.to_string(), "Warehouse with id 20 is already assigned to user with id 10");

    }

}