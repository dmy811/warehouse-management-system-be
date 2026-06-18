use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::models::{Rack, RackWithStats};

#[derive(Debug, Serialize)]
pub struct RackResponse {
    pub id: i64,
    pub warehouse_id: i64,
    pub code: String,
    pub zone: Option<String>,       
    pub level: Option<i32>,          
    pub description: Option<String>,
    pub capacity: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

impl From<Rack> for RackResponse {
    fn from(r: Rack) -> Self {
        Self {
            id: r.id,
            warehouse_id: r.warehouse_id,
            code: r.code,
            zone: r.zone,
            level: r.level,
            description: r.description,
            capacity: r.capacity,
            created_at: r.created_at,
            updated_at: r.updated_at
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RackSummary {
    pub id: i64,
    pub warehouse_id: i64,
    pub code: String,
    pub zone: Option<String>,       
    pub level: Option<i32>,          
    pub description: Option<String>,
    pub capacity: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub used_capacity: i64,
    pub total_products: i64,
}

impl From<RackWithStats> for RackSummary {
    fn from(r: RackWithStats) -> Self {
        Self {
            id: r.id,
            warehouse_id: r.warehouse_id,
            code: r.code,
            zone: r.zone,
            level: r.level,
            description: r.description,
            capacity: r.capacity,
            created_at: r.created_at,
            updated_at: r.updated_at,
            used_capacity: r.used_capacity,
            total_products: r.total_products
        }
    }
}