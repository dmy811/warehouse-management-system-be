use axum::{Extension, Json, extract::State, response::IntoResponse};
use serde_json::Value;
use validator::Validate;

use crate::{dtos::{AuthResponse, LoginRequest, UserResponse, auth_dto::UpdatePasswordRequest, user_dto::UpdateUserRequest}, errors::{AppError, AppResult}, middlewares::AuthUser, response::ApiResponse, state::AppState};

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>
) -> AppResult<impl IntoResponse> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
 
    let result: AuthResponse = state.services.auth.login(req).await?;
 
    Ok(ApiResponse::ok("Login successful", result))
}

pub async fn get_profile(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>
) -> AppResult<impl IntoResponse> {
    let user: UserResponse = state.services.auth.get_profile(auth_user.id).await?;

    Ok(ApiResponse::ok("User retrieved", user))
}

pub async fn update_profile(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<UpdateUserRequest>,
) -> AppResult<impl IntoResponse> {
    req.validate_is_empty()?;
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let result: UserResponse = state.services.auth.update_profile(auth_user.id, req).await?;

    Ok(ApiResponse::ok("Update successfull", result))
}

pub async fn change_profile_password(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<UpdatePasswordRequest>,
) -> AppResult<impl IntoResponse> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    
    state.services.auth.update_profile_password(auth_user.id, req).await?;

    Ok(ApiResponse::ok("Update password successfull", Value::Null))
}