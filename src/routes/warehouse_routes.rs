use axum::{Router, middleware, routing::{delete, get, patch, post}};

use crate::{handlers::warehouse_handler, middlewares::auth_middleware, state::AppState};

pub fn warehouse_routes() -> Router<AppState> {
    Router::new()
        .route("/warehouses", get(warehouse_handler::list))
        .route("/warehouses", post(warehouse_handler::create))
        .route("/warehouses/:id", get(warehouse_handler::get_by_id))
        .route("/warehouses/:id", patch(warehouse_handler::update))
        .route("/warehouses/:id", delete(warehouse_handler::delete))
        .route_layer(middleware::from_fn_with_state(
            AppState::dummy(),
            auth_middleware
        ))
}