use axum::{Extension, Json, extract::{Path, State}, response::IntoResponse};
use serde_json::Value;
use validator::Validate;

use crate::{constants::permissions, dtos::role_dto::{CreateRoleRequest, UpdateRoleRequest}, errors::{AppError, AppResult}, middlewares::{AuthUser, require_roles}, response::ApiResponse, state::AppState};

pub async fn list_all_roles(
    State(state): State<AppState>,
    Extension(_auth_user): Extension<AuthUser>
) -> AppResult<impl IntoResponse> {
    let roles = state.services.role.get_all_roles().await?;
    Ok(ApiResponse::ok("Roles retrieved", roles))
}

pub async fn get_role_by_id(
    State(state): State<AppState>,
    Extension(_auth_user): Extension<AuthUser>,
    Path(id): Path<i64>
) -> AppResult<impl IntoResponse> {
    let role = state.services.role.get_role_by_id(id).await?;
    Ok(ApiResponse::ok("Role retrieved", role))
}

pub async fn create_role(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateRoleRequest>
) -> AppResult<impl IntoResponse> {
    require_roles(permissions::CAN_MANAGE_USERS)(auth_user.clone())?;

    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let role = state.services.role.create_roles(&req.name).await?;

    Ok(ApiResponse::created("Role created", role))
}

pub async fn update_role(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateRoleRequest>
) -> AppResult<impl IntoResponse> {
    require_roles(permissions::CAN_MANAGE_USERS)(auth_user.clone())?;

    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let role = state.services.role.update_roles(id, req.name.as_deref()).await?;

    Ok(ApiResponse::ok("Role updated", role))
}

pub async fn delete_role(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i64>
) -> AppResult<impl IntoResponse> {
    require_roles(permissions::CAN_MANAGE_USERS)(auth_user.clone())?;

    state.services.role.delete_roles(id).await?;
    Ok(ApiResponse::ok("Role deleted", Value::Null))
}