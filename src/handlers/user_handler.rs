use std::collections::HashMap;

use axum::{Extension, Json, extract::{Path, Query, State}, response::IntoResponse};
use serde_json::Value;
use validator::Validate;

use crate::{constants::permissions, dtos::{UserResponse, user_dto::{CreateUserRequest, DeleteUserQuery, UpdateUserRequest}}, errors::{AppError, AppResult}, middlewares::{AuthUser, require_roles}, response::{ApiResponse, ListQuery, PaginatedResponse}, state::AppState};

pub async fn create_user(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateUserRequest>
) -> AppResult<impl IntoResponse> {
    require_roles(permissions::CAN_MANAGE_USERS)(auth_user.clone())?;
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let result: UserResponse = state.services.user.create_user(req).await?;

    Ok(ApiResponse::created("Create user successful", result))
}

pub async fn list_all_users(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ListQuery>
) -> AppResult<impl IntoResponse> {
    require_roles(permissions::CAN_MANAGE_MASTER)(auth_user.clone())?;
    let result: PaginatedResponse<UserResponse> = state.services.user.list_all_users(query).await?;
    Ok(result)
}

pub async fn find_user_by_id(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i64> 
) -> AppResult<impl IntoResponse> {
    require_roles(permissions::CAN_MANAGE_USERS)(auth_user.clone())?;

    let result = state.services.user.find_user_by_id(id).await?;

    Ok(ApiResponse::ok("User retrieved", result))
}

pub async fn update_user(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateUserRequest>,
) -> AppResult<impl IntoResponse> {
    require_roles(permissions::CAN_MANAGE_USERS)(auth_user.clone())?;
    req.validate_is_empty()?;
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let result: UserResponse = state.services.user.update_user(id, req).await?;

    Ok(ApiResponse::ok("Update user successfull", result))
}

pub async fn delete_user(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i64>,
    Query(query): Query<DeleteUserQuery>
) -> AppResult<impl IntoResponse> {
    require_roles(permissions::CAN_MANAGE_USERS)(auth_user.clone())?;

    let mode = query.mode.as_deref().unwrap_or("soft");

    let message = match mode {
        "hard" => {
            state.services.user.user_hard_delete(id).await?;
            "Permanently deleted user successfully"
        }
        _ => {
            state.services.user.user_soft_delete(id).await?;
            "Soft deleted user successfully"
        }
    };
    

    Ok(ApiResponse::ok(message, Value::Null))
}