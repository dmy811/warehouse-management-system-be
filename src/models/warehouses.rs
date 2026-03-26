use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Warehouse {
    pub id: i64,
    pub name: String,
    pub address: String,
    pub photo: Option<String>,
    pub phone: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

#[derive(Debug, Clone, FromRow)]
pub struct WarehouseWithStats {
    pub id: i64,
    pub name: String,
    pub address: String,
    pub photo: Option<String>,
    pub phone: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub total_products: Option<i64>, // total distinct product stored in this warehouse
    pub total_racks: Option<i64> // total rack count
}