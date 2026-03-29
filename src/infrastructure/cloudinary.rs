use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::multipart;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tracing::debug;

use crate::errors::AppError;
use crate::infrastructure::config::CloudinaryConfig;
use crate::constants::file_upload::{MAX_FILE_SIZE, ALLOWED_MIME_TYPES, ALLOWED_EXTENSIONS};

pub struct UploadOptions {
    pub folder: &'static str,
    pub public_id: Option<String>,
    pub optimize: bool
}

impl UploadOptions {
    pub fn for_user(user_id: i64) -> Self {
        Self {
            folder: "wms/users",
            public_id: Some(format!("user_{}", user_id)),
            optimize: true
        }
    }

    pub fn for_warehouse(warehouse_id: i64) -> Self {
        Self {
            folder: "wms/warehouses",
            public_id: Some(format!("warehouse_{}", warehouse_id)),
            optimize: true
        }
    }

    pub fn for_product(product_id: i64) -> Self {
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

impl CloudinaryClient {
    pub fn new(config: CloudinaryConfig) -> Self {
        Self {
            config,
            http: reqwest::Client::new()
        }
    }

    pub async fn upload(
        &self,
        file_bytes: Vec<u8>,
        file_name: &str,
        content_type: &str,
        options: UploadOptions
    ) -> Result<CloudinaryUploadResponse, AppError> {
        // --- Validation ---
        if file_bytes.len() > MAX_FILE_SIZE {
            return Err(AppError::Validation(format!(
                "File size {}MB exceeds the {}MB limit",
                file_bytes.len() / 1024 / 1024,
                MAX_FILE_SIZE / 1024 / 1024
            )));
        }

        if !ALLOWED_MIME_TYPES.contains(&content_type) {
            return Err(AppError::Validation(format!(
                "File type '{}' is not allowed. Allowed types: {}",
                content_type,
                ALLOWED_MIME_TYPES.join(", "
            ))));
        }

        // sama kayak .split() dengan .last(), kenapa pakai rsplit() dan next() karena jauh lebih cepat kompleksitas waktunya
        let ext = file_name
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_lowercase();

        if !ALLOWED_EXTENSIONS.contains(&&ext.as_str()) {
            return Err(AppError::Validation(format!(
                "File extension '.{}' is not allowed. Allowed: {}",
                ext,
                ALLOWED_EXTENSIONS.join(", ")
            )));
        }

        // -- Build signed params ---

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("System time error: {}", e)))?
            .as_secs()
            .to_string();

        let mut params: Vec<(String, String)> = vec![
            ("folder".to_string(), options.folder.to_string()),
            ("timestamp".to_string(), timestamp.clone())
        ];

        if let Some(pid) = &options.public_id {
            params.push(("public_id".to_string(), pid.clone()));
        }

        if options.optimize {
            params.push(
                ( "eager".to_string(), "c_fill,w_800,h_800,q_auto,f_auto".to_string()) // oficiall from cloudinary for transforming
            );
        }

        params.sort_by(|a, b| a.0.cmp(&b.0));

        let params_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        let signature = Self::sha256_hex(&format!("{}{}", params_string, self.config.api_secret));
        // "eager=...&folder=...&public_id=...SECRET_KEY"

        debug!(
            folder = options.folder,
            file_name = file_name,
            bytes = file_bytes.len(),
            "Uploading file to Cloudinary"
        );
 
        // --- Build multipart form ---
 
        let file_part = multipart::Part::bytes(file_bytes)
            .file_name(file_name.to_string())
            .mime_str(content_type)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid MIME type: {}", e)))?;
 
        let mut form = multipart::Form::new()
            .text("api_key", self.config.api_key.clone())
            .text("timestamp", timestamp)
            .text("signature", signature)
            .text("folder", options.folder.to_string())
            .part("file", file_part);
 
        if let Some(pid) = options.public_id {
            form = form.text("public_id", pid);
        }
 
        if options.optimize {
            form = form.text("eager", "c_fill,w_800,h_800,q_auto,f_auto");
        }
 
        // --- Send request ---
 
        let response = self
            .http
            .post(&self.config.upload_url())
            .multipart(form)
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Cloudinary request failed: {}", e)))?;
 
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to read Cloudinary response: {}", e)))?;
 
        if !status.is_success() {
            let message = serde_json::from_str::<CloudinaryErrorResponse>(&body)
                .map(|e| e.error.message)
                .unwrap_or_else(|_| format!("HTTP {}", status));
 
            return Err(AppError::Internal(anyhow::anyhow!(
                "Cloudinary upload failed: {}",
                message
            )));
        }
 
        serde_json::from_str::<CloudinaryUploadResponse>(&body).map_err(|e| {
            AppError::Internal(anyhow::anyhow!(
                "Failed to parse Cloudinary response: {}",
                e
            ))
        })
    }

    pub async fn delete(&self, public_id: &str) -> Result<(), AppError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?
            .as_secs()
            .to_string();
 
        let to_sign = format!(
            "public_id={}&timestamp={}{}",
            public_id, timestamp, self.config.api_secret
        );
        let signature = Self::sha256_hex(&to_sign);
 
        let destroy_url = format!(
            "https://api.cloudinary.com/v1_1/{}/image/destroy",
            self.config.cloud_name
        );
 
        self.http
            .post(&destroy_url)
            .multipart(
                multipart::Form::new()
                    .text("public_id", public_id.to_string())
                    .text("api_key", self.config.api_key.clone())
                    .text("timestamp", timestamp)
                    .text("signature", signature),
            )
            .send()
            .await
            .map_err(|e| {
                AppError::Internal(anyhow::anyhow!("Cloudinary delete failed: {}", e))
            })?;
 
        Ok(())
    }

    fn sha256_hex(input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        hex::encode(hasher.finalize())
    }
}