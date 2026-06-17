use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Rack {
    pub id: i64,
    pub warehouse_id: i64,
    pub code: String,
    pub zone: Option<String>,       
    pub level: Option<i32>,          
    pub description: Option<String>,
    pub capacity: Option<i64>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}


#[derive(Debug, Clone, FromRow)]
pub struct RackWithStats {
    pub id: i64,
    pub warehouse_id: i64,
    pub code: String,
    pub zone: Option<String>,       
    pub level: Option<i32>,          
    pub description: Option<String>,
    pub capacity: Option<i64>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub used_capacity: i64,
    pub total_products: i64,
}