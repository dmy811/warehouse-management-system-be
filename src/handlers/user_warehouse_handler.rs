use axum::{Extension, extract::{State, Path}, Json, response::IntoResponse};
use validator::Validate;

use crate::{constants::permissions, dtos::user_warehouse_dto::AssignWarehouseRequest, errors::{AppError, AppResult}, middlewares::{AuthUser, require_roles}, response::ApiResponse, state::AppState};

pub async fn assign_warehouse_to_user(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(user_id): Path<i64>,
    Json(req): Json<AssignWarehouseRequest>
) -> AppResult<impl IntoResponse> {
    require_roles(permissions::CAN_MANAGE_MASTER)(auth_user.clone())?;

    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    state.services.user_warehouse.assign_warehouse_to_user(user_id, req.warehouse_id).await?;

    Ok(ApiResponse::ok("Successfully assigned warehouse", serde_json::json!({
        "user_id": user_id,
        "warehouse_id": req.warehouse_id
    })))
}