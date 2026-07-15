use axum::{Router, routing::post};

use crate::{handlers::user_warehouse_handler, state::AppState};

pub fn user_warehouse_routes() -> Router<AppState> {
    Router::new()
        .route("users/{user_id}/warehouses", post(user_warehouse_handler::assign_warehouse_to_user))
}