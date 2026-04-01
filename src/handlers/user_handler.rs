use axum::{Extension, Json, extract::{Path, State}, response::IntoResponse};
use validator::Validate;

use crate::{constants::permissions, dtos::{UserResponse, user_dto::{CreateUserRequest, UpdateUserRequest}}, errors::{AppError, AppResult}, middlewares::{AuthUser, require_roles}, response::ApiResponse, state::AppState};

pub async fn create(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateUserRequest>
) -> AppResult<impl IntoResponse> {
    require_roles(permissions::CAN_MANAGE_USERS)(auth_user.clone())?;
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let result: UserResponse = state.services.user.create(req).await?;

    Ok(ApiResponse::created("Create user successful", result))
}

pub async fn update(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateUserRequest>,
) -> AppResult<impl IntoResponse> {
    require_roles(permissions::CAN_MANAGE_USERS)(auth_user.clone())?;
    req.validate_is_empty()?;
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let result: UserResponse = state.services.user.update(id, req).await?;

    Ok(ApiResponse::ok("Update user successfull", result))
}