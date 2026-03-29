use axum::{Extension, extract::{Multipart, Path, State}, response::IntoResponse};
use tracing::info;

use crate::{constants::{file_upload::{ALLOWED_MIME_TYPES, MAX_FILE_SIZE, PHOTO_FIELD}, roles}, errors::{AppError, AppResult}, infrastructure::cloudinary::UploadOptions, middlewares::{AuthUser, require_roles}, response::ApiResponse, state::AppState};

async fn extract_photo(
    multipart: &mut Multipart
) -> AppResult<(Vec<u8>, String, String)> {
    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::Validation(format!("Invalid multipart data: {}", e)))? {
        if field.name().unwrap_or("") != PHOTO_FIELD {
            continue;
        }

        let file_name = field
            .file_name()
            .map(|f| f.to_string())
            .unwrap_or_else(|| "upload.jpg".to_string());

        let content_type = field
            .content_type()
            .map(|ct| ct.to_string())
            .unwrap_or_else(|| "image/jpeg".to_string());

        if !ALLOWED_MIME_TYPES.contains(&content_type.as_str()) {
            return Err(AppError::Validation(format!(
                "File type '{}' is not allowed. Allowed types: {}",
                content_type,
                ALLOWED_MIME_TYPES.join(", "
            ))));
        }

        let bytes = field
            .bytes()
            .await
            .map_err(|e| AppError::Validation(format!("Failed to read file: {}", e)))?;

        if bytes.is_empty() {
            return Err(AppError::Validation("File is empty".to_string()));
        }

        if bytes.len() > MAX_FILE_SIZE {
            return Err(AppError::Validation(format!(
                "File size {}MB exceeds the {}MB limit",
                bytes.len() / 1024 / 1024,
                MAX_FILE_SIZE / 1024 / 1024
            )));
        }

        return Ok((bytes.to_vec(), file_name, content_type));
    }

    Err(AppError::Validation(format!(
        "Missing '{}' field in multipart form",
        PHOTO_FIELD
    )))
}

pub async fn upload_user_photo(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    mut multipart: Multipart
) -> AppResult<impl IntoResponse> {
    let (bytes, file_name, content_type) = extract_photo(&mut multipart).await?;

    let upload = state
        .cloudinary
        .upload(bytes, &file_name, &content_type, UploadOptions::for_user(auth_user.id))
        .await?;

    state
        .services
        .auth
        .update_photo(auth_user.id, &upload.secure_url)
        .await?;

    info!(
        user_id = auth_user.id,
        url = %upload.secure_url,
        bytes = upload.bytes,
        "User photo uploaded"
    );

    Ok(ApiResponse::ok(
        "Photo uploaded successfully",
        serde_json::json!({
            "photo_url": upload.secure_url
        })
    ))
}

pub async fn delete_user_photo(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> AppResult<impl axum::response::IntoResponse> {
    state.services.auth.delete_photo(auth_user.id).await?;
 
    info!(user_id = auth_user.id, "User photo deleted");
 
    Ok(ApiResponse::no_content())
}

pub async fn upload_warehouse_photo(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(warehouse_id): Path<i64>,
    mut multipart: Multipart
) -> AppResult<impl IntoResponse> {
    require_roles(&[roles::ADMIN, roles::MANAGER])(auth_user.clone())?;

    let (bytes, file_name, content_type) = extract_photo(&mut multipart).await?;

    let upload = state
        .cloudinary
        .upload(bytes, &file_name, &content_type, UploadOptions::for_warehouse(warehouse_id))
        .await?;

    state
        .services
        .warehouse
        .update_photo(warehouse_id, &upload.secure_url, auth_user.id)
        .await?;

    info!(
        warehouse_id = warehouse_id,
        user_id = auth_user.id,
        url = %upload.secure_url,
        bytes = upload.bytes,
        "Warehouse photo uploaded"
    );

    Ok(ApiResponse::ok(
        "Photo uploaded successfully",
        serde_json::json!({
            "photo_url": upload.secure_url
        })
    ))
}

pub async fn delete_warehouse_photo(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(warehouse_id): Path<i64>,
) -> AppResult<impl axum::response::IntoResponse> {
    require_roles(&[roles::ADMIN, roles::MANAGER])(auth_user.clone())?;
 
    state
        .services
        .warehouse
        .delete_photo(warehouse_id, auth_user.id)
        .await?;
 
    info!(
        warehouse_id = warehouse_id,
        actor_id = auth_user.id,
        "Warehouse photo deleted"
    );
 
    Ok(ApiResponse::no_content())
}