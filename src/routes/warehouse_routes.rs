use axum::{Router, routing::{delete, get, patch, post}};

use crate::{handlers::{upload_handler, warehouse_handler}, state::AppState};

pub fn warehouse_routes() -> Router<AppState> {
    Router::new()
        .route("/warehouses", get(warehouse_handler::list_all_warehouses))
        .route("/warehouses", post(warehouse_handler::create_warehouse))
        .route("/warehouses/{id}", get(warehouse_handler::get_warehouse_by_id))
        .route("/warehouses/{id}", patch(warehouse_handler::update_warehouse))
        .route("/warehouses/{id}", delete(warehouse_handler::delete_warehouse))
        .route("/warehouses/{id}/photo", post(upload_handler::upload_warehouse_photo))
        .route("/warehouses/{id}/photo", delete(upload_handler::delete_warehouse_photo))
    }