use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{constants::PHONE_REGEX, models::{Warehouse, WarehouseWithStats}};

#[derive(Debug, Deserialize, Validate)]
pub struct CreateWarehouseRequest {
    #[validate(length(min = 2, max = 100, message = "Name must be between 2 and 100 characters"))]
    pub name: String,

    #[validate(length(min = 5, max = 500, message = "Address must be between 5 and 500 characters"))]
    pub address: String,

    #[validate(length(min = 8, max = 20, message = "Phone must be between 8 and 20 characters!"), regex(path = "*PHONE_REGEX", message = "Invalid phone number format"))]
    pub phone: Option<String>,

    pub photo: Option<String>
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateWarehouseRequest {
    #[validate(length(min = 2, max = 100, message = "Name must be between 2 and 100 characters"))]
    pub name: Option<String>,

    #[validate(length(min = 5, max = 500, message = "Address must be between 5 and 500 characters"))]
    pub address: Option<String>,

    #[validate(length(min = 8, max = 20, message = "Phone must be between 8 and 20 characters!"), regex(path = "*PHONE_REGEX", message = "Invalid phone number format"))]
    pub phone: Option<String>,

    pub photo: Option<String>
}

impl UpdateWarehouseRequest {
    // returns true if at least one field is provided
    pub fn is_empty(&self) -> bool {
        self.name.is_none() && self.address.is_none() && self.phone.is_none() && self.photo.is_none()
    }
}

#[derive(Debug, Deserialize)]
pub struct ListWarehouseQuery {
    #[serde(default = "default_page")] // page number
    pub page: i64,

    #[serde(default = "default_per_page")] // items per page 
    pub per_page: i64,

    pub search: Option<String>, // search by name or address (case-insensitive, partial match)

    #[serde(default = "default_sort_by")] // sort field: name | created_at | updated_at
    pub sort_by: String,

    #[serde(default = "default_sort_order")] // sort direction: asc | desc
    pub sort_order: String,
}

fn default_page() -> i64 { 1 }
fn default_per_page() -> i64 { 20 }
fn default_sort_by() -> String { "created_at".to_string() }
fn default_sort_order() -> String { "desc".to_string() }

impl ListWarehouseQuery {
    pub fn offset(&self) -> i64 {
        (self.page.max(1) - 1) * self.per_page()
    }

    pub fn per_page(&self) -> i64 {
        self.per_page.clamp(1, 100)
    }

    pub fn sort_column(&self) -> &str {
        match self.sort_by.as_str() {
            "name" => "w.name",
            "updated_at" => "w.updated_at",
            _ => "w.created_at" // default
        }
    }

    pub fn sort_direction(&self) -> &str {
        match self.sort_order.to_lowercase().as_str() {
            "asc" => "ASC",
            _ => "DESC", // default
        }
    }
}

#[derive(Debug, Serialize)]
pub struct WarehouseResponse {
    pub id: i64,
    pub name: String,
    pub address: String,
    pub photo: Option<String>,
    pub phone: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

impl From<Warehouse> for WarehouseResponse {
    fn from(w: Warehouse) -> Self {
        Self {
            id: w.id,
            name: w.name,
            address: w.address,
            photo: w.photo,
            phone: w.phone,
            created_at: w.created_at,
            updated_at: w.updated_at
        }
    }
}

#[derive(Debug, Serialize)]
pub struct WarehouseSummary {
    pub id: i64,
    pub name: String,
    pub address: String,
    pub photo: Option<String>,
    pub phone: Option<String>,
    pub total_products: i64,
    pub total_racks: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
 
impl From<WarehouseWithStats> for WarehouseSummary {
    fn from(w: WarehouseWithStats) -> Self {
        Self {
            id: w.id,
            name: w.name,
            address: w.address,
            photo: w.photo,
            phone: w.phone,
            total_products: w.total_products.unwrap_or(0),
            total_racks: w.total_racks.unwrap_or(0),
            created_at: w.created_at,
            updated_at: w.updated_at,
        }
    }
}