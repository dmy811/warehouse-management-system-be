use axum::{Extension, Json, extract::State, response::IntoResponse};
use validator::Validate;

use crate::{constants::permissions, dtos::{UserResponse, user_dto::CreateUserRequest}, errors::{AppError, AppResult}, middlewares::{AuthUser, require_roles}, response::ApiResponse, state::AppState};

pub async fn create(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateUserRequest>
) -> AppResult<impl IntoResponse> {
    require_roles(permissions::CAN_MANAGE_USERS)(auth_user.clone())?;
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let result: UserResponse = state.services.user.create(req).await?;

    Ok(ApiResponse::ok("Create user successful", result))
}