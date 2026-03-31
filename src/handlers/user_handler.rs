use axum::{Json, extract::State, response::IntoResponse};
use validator::Validate;

use crate::{dtos::{UserResponse, user_dto::CreateUserRequest}, errors::{AppError, AppResult}, response::ApiResponse, state::AppState};

pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>
) -> AppResult<impl IntoResponse> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let result: UserResponse = state.services.user.create(req).await?;

    Ok(ApiResponse::ok("Create user successful", result))
}