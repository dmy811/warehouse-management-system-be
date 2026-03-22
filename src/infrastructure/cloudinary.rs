use serde::Deserialize;

use crate::infrastructure::config::CloudinaryConfig;

pub const MAX_FILE_SIZE: usize = 5 * 1024 * 1024; // 5MB
pub const ALLOWED_MIME_TYPES: &[&str] = &["image/jpeg", "image/png", "image/webp"];
pub const ALLOWED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp"];

pub struct UploadOptions {
    pub folder: &'static str,
    pub public_id: Option<String>,
    pub optimize: bool
}

impl UploadOptions {
    pub fn for_user(user_id: i32) -> Self {
        Self {
            folder: "wms/users",
            public_id: Some(format!("user_{}", user_id)),
            optimize: true
        }
    }

    pub fn for_warehouse(warehouse_id: i32) -> Self {
        Self {
            folder: "wms/warehouses",
            public_id: Some(format!("warehouse_{}", warehouse_id)),
            optimize: true
        }
    }

    pub fn for_product(product_id: i32) -> Self {
        Self {
            folder: "wms/products",
            public_id: Some(format!("product_{}", product_id)),
            optimize: true
        }
    }
}

// response from cloudinary after successful upload
#[derive(Debug, Deserialize)]
pub struct CloudinaryUploadResponse {
    pub secure_url: String,
    pub public_id: String,
    pub format: String,
    pub bytes: u64,
    pub width: Option<u32>,
    pub height: Option<u32>
}

#[derive(Debug, Deserialize)]
struct CloudinaryErrorResponse {
    error: CloudinaryErrorDetail
}

#[derive(Debug, Deserialize)]
struct CloudinaryErrorDetail {
    message: String,
}

pub struct CloudinaryClient {
    config: CloudinaryConfig,
    http: reqwest::Client
}