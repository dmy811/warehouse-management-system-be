use std::sync::Arc;

use async_trait::async_trait;

use tracing::info;

use crate::{errors::{AppError, AppResult}, repositories::{WarehouseRepositoryTrait, user_repository::UserRepositoryTrait, user_warehouse_repository::UserWarehouseRepositoryTrait}};

#[async_trait]
pub trait UserWarehouseServiceTrait: Send + Sync {
    async fn assign_warehouse_to_user(&self, user_id: i64, warehouse_id: i64) -> AppResult<()>;
}

pub struct UserWarehouseService <UW: UserWarehouseRepositoryTrait, U: UserRepositoryTrait, W: WarehouseRepositoryTrait> {
    pub repo: Arc<UW>,
    pub user_repo: Arc<U>,
    pub warehouse_repo: Arc<W>
}

impl <UW: UserWarehouseRepositoryTrait, U: UserRepositoryTrait, W: WarehouseRepositoryTrait> UserWarehouseService<UW, U, W> {
    pub fn new(repo: Arc<UW>, user_repo: Arc<U>, warehouse_repo: Arc<W>) -> Self {
        Self {
            repo,
            user_repo,
            warehouse_repo
        }
    }
}

#[async_trait]
impl <UW: UserWarehouseRepositoryTrait, U: UserRepositoryTrait, W: WarehouseRepositoryTrait> UserWarehouseServiceTrait for UserWarehouseService<UW, U, W> {
    async fn assign_warehouse_to_user(&self, user_id: i64, warehouse_id: i64) -> AppResult<()>{
        self.user_repo
            .find_user_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User with id {}", user_id)))?;
        
        self.warehouse_repo
            .find_warehouse_by_id(warehouse_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Warehouse with id {}", warehouse_id)))?;

        if self.repo.check_assign(user_id, warehouse_id).await? {
            return Err(AppError::Conflict(format!(
                "Warehouse with id {} is already assigned to user with id {}",
                warehouse_id, user_id
            )))
        }

        self.repo.assign_warehouse_to_user(user_id, warehouse_id).await?;

        info!(user_id, warehouse_id, "Warehouse successfully assigned to user");

        Ok(())
    }
}

// #[tokio::test]
//     async fn test_assign_warehouse_success() {
//         let mut mock_repo = MockWarehouseRepositoryTrait::new();
//         mock_repo
//             .expect_check_user_existing()
//             .with(eq(10))
//             .times(1)
//             .returning(|_| Ok(true));

//         mock_repo
//             .expect_find_warehouse_by_id()
//             .with(eq(20))
//             .times(1)
//             .returning(|_| Ok(Some(mock_warehouse_with_stats(20, "warehouse", "Jl jambu"))));

//         mock_repo
//             .expect_check_existing_warehouse_in_user()
//             .with(eq(10), eq(20))
//             .times(1)
//             .returning(|_, _| Ok(false));

//         mock_repo
//             .expect_assign_warehouse_to_user()
//             .with(eq(10), eq(20))
//             .times(1)
//             .returning(|_,_| Ok(()));

//         let service = setup_service(mock_repo);
//         let result = service.assign_warehouse_to_user(10, 20).await;

//         assert!(result.is_ok(), "Expected to successfull assigned warehouse to user");

//     }
    
//     #[tokio::test]
//     async fn test_assign_warehouse_user_not_exists() {
//         let mut mock_repo = MockWarehouseRepositoryTrait::new();
//         mock_repo
//             .expect_check_user_existing()
//             .with(eq(10))
//             .times(1)
//             .returning(|_| Ok(false));

//         let service = setup_service(mock_repo);
//         let result = service.assign_warehouse_to_user(10, 20).await;

//         assert!(result.is_err(), "Expected to error user not exists");
//         let err = result.unwrap_err();
//         assert!(matches!(err, AppError::NotFound(_)), "The expected type is NotFound");
//         assert_eq!(err.to_string(), "User with id 10 not found");

//     }

//     #[tokio::test]
//     async fn test_assign_warehouse_warehouse_not_exists() {
//         let mut mock_repo = MockWarehouseRepositoryTrait::new();
//         mock_repo
//             .expect_check_user_existing()
//             .with(eq(10))
//             .times(1)
//             .returning(|_| Ok(true));

//         mock_repo
//             .expect_find_warehouse_by_id()
//             .with(eq(20))
//             .times(1)
//             .returning(|_| Ok(None));

//         let service = setup_service(mock_repo);
//         let result = service.assign_warehouse_to_user(10, 20).await;

//         assert!(result.is_err(), "Expected to error warehouse not exists");
//         let err = result.unwrap_err();
//         assert!(matches!(err, AppError::NotFound(_)), "The expected type is NotFound");
//         assert_eq!(err.to_string(), "Warehouse with id 20 not found");

//     }

//     #[tokio::test]
//     async fn test_assign_warehouse_warehouse_already_assigned_to_user() {
//         let mut mock_repo = MockWarehouseRepositoryTrait::new();
//         mock_repo
//             .expect_check_user_existing()
//             .with(eq(10))
//             .times(1)
//             .returning(|_| Ok(true));

//         mock_repo
//             .expect_find_warehouse_by_id()
//             .with(eq(20))
//             .times(1)
//             .returning(|_| Ok(Some(mock_warehouse_with_stats(20, "warehouse", "Jl jambu"))));

//         mock_repo
//             .expect_check_existing_warehouse_in_user()
//             .with(eq(10), eq(20))
//             .times(1)
//             .returning(|_, _| Ok(true));

//         let service = setup_service(mock_repo);
//         let result = service.assign_warehouse_to_user(10, 20).await;

//         assert!(result.is_err(), "Expected to error warehouse not exists");
//         let err = result.unwrap_err();
//         assert!(matches!(err, AppError::Conflict(_)), "The expected type is Conflict");
//         assert_eq!(err.to_string(), "Warehouse with id 20 is already assigned to user with id 10");

//     }