use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{validators::common::PHONE_ID_REGEX, errors::AppError, models::{Warehouse, WarehouseWithStats}};

#[derive(Debug, Deserialize, Validate)]
pub struct CreateWarehouseRequest {
    #[validate(length(min = 2, max = 100, message = "Name must be between 2 and 100 characters"))]
    pub name: String,

    #[validate(length(min = 5, max = 500, message = "Address must be between 5 and 500 characters"))]
    pub address: String,

    #[validate(length(min = 10, max = 15, message = "Phone must be between 10 and 15 digits!"), regex(path = "*PHONE_ID_REGEX", message = "Phone must be a valid Indonesian number (+62xxx or 08xxx)"))]
    pub phone: Option<String>,

    pub photo: Option<String>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateWarehouseRequest {
    // #[validate(length(min = 2, max = 100, message = "Name must be between 2 and 100 characters"))]
    pub name: Option<String>,

    // #[validate(length(min = 5, max = 500, message = "Address must be between 5 and 500 characters"))]
    pub address: Option<String>,

    // #[validate(length(min = 10, max = 15, message = "Phone must be between 10 and 15 digits!"), regex(path = "*PHONE_ID_REGEX", message = "Phone must be a valid Indonesian number (+62xxx or 08xxx)"))]
    pub phone: Option<String>,

    pub photo: Option<String>
}

impl UpdateWarehouseRequest {
    // returns true if at least one field is provided
    pub fn is_empty(&self) -> bool {
        self.name.is_none() && self.address.is_none() && self.phone.is_none() && self.photo.is_none()
    }

    // custom validation
    pub fn validate(&self) -> Result<(), AppError> {
        let mut errors: Vec<String> = Vec::new();

        if let Some(name) = &self.name {
            if name.len() < 2 || name.len() > 100 {
                errors.push("Name must be between 2 and 100 characters".to_string());
            }
        }

        if let Some(address) = &self.address {
            if address.len() < 5 || address.len() > 500 {
                errors.push("Address must be between 5 and 500 characters".to_string());
            }
        }

        if let Some(phone) = &self.phone {
            if !PHONE_ID_REGEX.is_match(phone) {
                errors.push("Phone must be a valid Indonesian number (+62xxx or 08xxx) and between 10 - 15 digits".to_string());
            }
        }

        if self.is_empty() {
            errors.push("At least one field must be provided for update".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(AppError::Validation(errors.join(", ")))
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