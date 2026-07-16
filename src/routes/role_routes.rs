use axum::{Router, routing::{delete, get, put, post}};

use crate::{handlers::role_handler, state::AppState};

pub fn role_routes() -> Router<AppState> {
    Router::new()
        .route("/roles", get(role_handler::list_all_roles))
        .route("/roles", post(role_handler::create_role))
        .route("/roles/{id}", get(role_handler::get_role_by_id))
        .route("/roles/{id}", put(role_handler::update_role))
        .route("/roles/{id}", delete(role_handler::delete_role))
}