use axum::{Extension, Json, extract::State, response::IntoResponse};
use validator::Validate;

use crate::{dtos::{AuthResponse, LoginRequest, RegisterRequest, UserResponse}, errors::{AppError, AppResult}, middlewares::AuthUser, response::ApiResponse, state::AppState};

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>
) -> AppResult<impl IntoResponse> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let result: AuthResponse = state.services.auth.register(req).await?;

    Ok(ApiResponse::created("Registration successful", result))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>
) -> AppResult<impl IntoResponse> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
 
    let result: AuthResponse = state.services.auth.login(req).await?;
 
    Ok(ApiResponse::ok("Login successful", result))
}

pub async fn me(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>
) -> AppResult<impl IntoResponse> {
    let user: UserResponse = state.services.auth.me(auth_user.id).await?;

    Ok(ApiResponse::ok("User retrieved", user))
}