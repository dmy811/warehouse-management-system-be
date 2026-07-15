use serde::{Deserialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct AssignWarehouseRequest {
    #[validate(range(min = 1, message = "Warehouse ID not valid"))]
    pub warehouse_id: i64
}