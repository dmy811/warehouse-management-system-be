use axum::{Extension, Json, extract::{Path, Query, State}, response::IntoResponse};
use validator::Validate;

use crate::{constants::permissions, dtos::{CreateWarehouseRequest, UpdateWarehouseRequest, WarehouseResponse, WarehouseSummary}, errors::{AppError, AppResult}, middlewares::{AuthUser, require_roles}, response::{ApiResponse, PaginatedResponse, ListQuery}, state::AppState};

pub async fn list_all_warehouses(
    State(state): State<AppState>,
    Extension(_auth_user): Extension<AuthUser>,
    Query(query): Query<ListQuery>
) -> AppResult<impl IntoResponse> {
    let result: PaginatedResponse<WarehouseSummary> = state.services.warehouse.list_all_warehouses(query).await?;
    Ok(result)
}

pub async fn get_warehouse_by_id(
    State(state): State<AppState>,
    Extension(_auth_user): Extension<AuthUser>,
    Path(id): Path<i64>
) -> AppResult<impl IntoResponse> {
    let warehouse: WarehouseResponse = state.services.warehouse.get_warehouse_by_id(id).await?;

    Ok(ApiResponse::ok("Warehouse retrieved", warehouse))
}

pub async fn create_warehouse(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateWarehouseRequest>,
) -> AppResult<impl IntoResponse> {
    require_roles(permissions::CAN_MANAGE_MASTER)(auth_user.clone())?;

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
) -> AppResult<impl axum::response::IntoResponse> {
    require_roles(permissions::CAN_MANAGE_MASTER)(auth_user.clone())?;
 
    req.validate()?;
 
    let warehouse: WarehouseResponse = state.services.warehouse.update_warehouse(id, req, auth_user.id).await?;
    Ok(ApiResponse::ok("Warehouse updated", warehouse))
}

pub async fn delete_warehouse(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> AppResult<impl axum::response::IntoResponse> {
    require_roles(permissions::CAN_MANAGE_USERS)(auth_user.clone())?;
 
    state.services.warehouse.delete_warehouse(id, auth_user.id).await?;
    Ok(ApiResponse::no_content())
}