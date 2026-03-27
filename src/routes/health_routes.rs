use axum::{Router, routing::get};

use crate::{handlers::health_handler, state::AppState};

pub fn health_routes() -> Router<AppState> {
    Router::new()
        .route("/ping", get(health_handler::ping))
        .route("/health", get(health_handler::db_health_check))
}