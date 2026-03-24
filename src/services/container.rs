use std::sync::Arc;

use sqlx::PgPool;

use crate::infrastructure::config::Config;

#[derive(Clone)]
pub struct ServiceContainer {
    // pub auth: Arc<dyn AuthServiceTrait>,
    // pub warehouse: Arc<dyn WarehouseServiceTrait>,
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
    pub fn new(db: &PgPool, config: &Arc<Config>) -> Self {
        // let auth_repo = Arc::new(AuthRepository::new(db.clone()));
        // let warehouse_repo = Arc::new(WarehouseRepository::new(db.clone()));
        // let product_repo = Arc::new(ProductRepository::new(db.clone()));
        // ...

        Self {
            // auth: Arc::new(AuthService::new(auth_repo, config.clone())),
            // warehouse: Arc::new(WarehouseService::new(warehouse_repo)),
            // product: Arc::new(ProductService::new(product_repo)),
            // // ...
        }
    }

    // pub fn dummy() -> Self {
    //     Self {
    //         auth: Arc::new(DummyAuthService),
    //         warehouse: Arc::new(DummyWarehouseService),
    //         product: Arc::new(DummyProductService),
    //         // ...
    //     }
    // }
}