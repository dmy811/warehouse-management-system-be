use std::sync::Arc;

use sqlx::PgPool;
use deadpool_redis::Pool as RedisPool;

use crate::{infrastructure::config::Config, repositories::{AuthRepository, WarehouseRepository, user_repository::UserRepository, user_warehouse_repository::UserWarehouseRepository}, services::{AuthService, AuthServiceTrait, WarehouseService, WarehouseServiceTrait, user_service::{UserService, UserServiceTrait}, user_warehouse_service::{UserWarehouseService, UserWarehouseServiceTrait}}};

#[derive(Clone)]
pub struct ServiceContainer {
    pub auth: Arc<dyn AuthServiceTrait>,
    pub warehouse: Arc<dyn WarehouseServiceTrait>,
    pub user: Arc<dyn UserServiceTrait>,
    pub user_warehouse: Arc<dyn UserWarehouseServiceTrait>
    // pub product: Arc<dyn ProductServiceTrait>,
    // pub category: Arc<dyn CategoryServiceTrait>,
    // pub supplier: Arc<dyn SupplierServiceTrait>,
    // pub customer: Arc<dyn CustomerServiceTrait>,
    // pub rack: Arc<dyn RackServiceTrait>,
    // pub inventory: Arc<dyn InventoryServiceTrait>,
    // pub goods_receipt: Arc<dyn GoodsReceiptServiceTrait>,
    // pub shipment: Arc<dyn ShipmentServiceTrait>,
    // pub transfer: Arc<dyn TransferServiceTrait>,
    // pub stock_movement: Arc<dyn StockMovementServiceTrait>,
}

impl ServiceContainer {
    pub fn new(db: &PgPool, config: &Arc<Config>, redis_pool: &Arc<RedisPool>) -> Self {
        let warehouse_repo = Arc::new(WarehouseRepository::new(db.clone()));
        let user_repo = Arc::new(UserRepository::new(db.clone()));
        let user_warehouse_repo = Arc::new(UserWarehouseRepository::new(db.clone()));
        // let product_repo = Arc::new(ProductRepository::new(db.clone()));
        // ...

        Self {
            auth: Arc::new(AuthService::new(user_repo.clone(), config.clone(), redis_pool.clone())),
            warehouse: Arc::new(WarehouseService::new(warehouse_repo.clone())),
            user: Arc::new(UserService::new(user_repo.clone())),
            user_warehouse: Arc::new(UserWarehouseService::new(user_warehouse_repo, user_repo.clone(), warehouse_repo.clone()))
            // product: Arc::new(ProductService::new(product_repo)),
            // // ...
        }
    }
}