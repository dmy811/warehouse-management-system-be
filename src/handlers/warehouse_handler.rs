use axum::{Extension, Json, extract::{Path, Query, State}, response::IntoResponse};
use serde_json::Value;
use validator::Validate;

use crate::{constants::permissions, dtos::{CreateWarehouseRequest, UpdateWarehouseRequest, WarehouseResponse, WarehouseSummary, warehouse_dto::DeleteWarehouseQuery}, errors::{AppError, AppResult}, middlewares::{AuthUser, require_roles}, response::{ApiResponse, ListQuery, PaginatedResponse}, state::AppState};

pub async fn list_all_warehouses(
    State(state): State<AppState>,
    Extension(_auth_user): Extension<AuthUser>,
    Query(query): Query<ListQuery>
) -> AppResult<impl IntoResponse> {
    let result: PaginatedResponse<WarehouseSummary> = state.services.warehouse.get_all_warehouses(query).await?;
    Ok(result)
}

pub async fn get_warehouse_by_id(
    State(state): State<AppState>,
    Extension(_auth_user): Extension<AuthUser>,
    Path(id): Path<i64>
) -> AppResult<impl IntoResponse> {
    let warehouse: WarehouseSummary = state.services.warehouse.get_warehouse_by_id(id).await?;

    Ok(ApiResponse::ok("Warehouse retrieved", warehouse))
}

pub async fn create_warehouse(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateWarehouseRequest>,
) -> AppResult<impl IntoResponse> {
    require_roles(permissions::CAN_MANAGE_USERS)(auth_user.clone())?;

    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let warehouse: WarehouseResponse = state.services.warehouse.create_warehouse(req, auth_user.id).await?;

    Ok(ApiResponse::created("Warehouse created", warehouse))
}

pub async fn update_warehouse(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateWarehouseRequest>,
) -> AppResult<impl IntoResponse> {
    require_roles(permissions::CAN_MANAGE_MASTER)(auth_user.clone())?;
 
    req.validate()?;
 
    let warehouse: WarehouseResponse = state.services.warehouse.update_warehouse(id, req, auth_user.id).await?;
    Ok(ApiResponse::ok("Warehouse updated", warehouse))
}

pub async fn delete_warehouse(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i64>,
    Query(query): Query<DeleteWarehouseQuery>
) -> AppResult<impl IntoResponse> {
    require_roles(permissions::CAN_MANAGE_USERS)(auth_user.clone())?;

    let mode = query.mode.as_deref().unwrap_or("soft");

    let message = match mode {
        "hard" => {
            state.services.warehouse.delete_warehouse_soft(id, auth_user.id).await?;
            "Warehouse permanenlty deleted successfully"
        }
        _ => {
            state.services.warehouse.delete_warehouse_hard(id, auth_user.id).await?;
            "Warehouse soft deleted successfully"
        }
    };

    Ok(ApiResponse::ok(message, Value::Null))
}