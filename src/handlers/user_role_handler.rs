use axum::{Extension, Json, extract::{Path, State}, response::IntoResponse};
use validator::Validate;

use crate::{constants::permissions, dtos::user_role_dto::AssignRoleRequest, errors::{AppError, AppResult}, middlewares::{AuthUser, require_roles}, response::ApiResponse, state::AppState};


pub async fn assign_role_to_user (
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(user_id): Path<i64>,
    Json(req): Json<AssignRoleRequest>
) -> AppResult<impl IntoResponse> {
    require_roles(permissions::CAN_MANAGE_USERS)(auth_user.clone())?;

    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    state.services.user_role.assign_role_to_user(user_id, req.role_id).await?;

    Ok(ApiResponse::ok("Successfully assigned role", serde_json::json!({
        "user_id": user_id,
        "role_id": req.role_id
    })))
}