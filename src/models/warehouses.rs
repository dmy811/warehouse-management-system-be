use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Warehouse {
    pub id: i64,
    pub name: String,
    pub address: String,
    pub photo: Option<String>,
    pub phone: Option<String>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWarehouseRequest {
    pub name: String,
    pub address: String,
    pub photo: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWarehouseRequest {
    pub name: Option<String>,
    pub address: Option<String>,
    pub photo: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WarehouseResponse {
    pub id: i64,
    pub name: String,
    pub address: String,
    pub photo: Option<String>,
    pub phone: Option<String>,
}

impl From<Warehouse> for WarehouseResponse {
    fn from(warehouse: Warehouse) -> Self {
        Self {
            id: warehouse.id,
            name: warehouse.name,
            address: warehouse.address,
            photo: warehouse.photo,
            phone: warehouse.phone,
        }
    }
}