use axum::{Router, routing::{delete, get, patch, post}};

use crate::{handlers::user_handler, state::AppState};

pub fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/users", get(user_handler::list_all_users))
        .route("/users", post(user_handler::create_user))
        .route("/users/{id}", get(user_handler::find_user_by_id))
        .route("/users/{id}", patch(user_handler::update_user))
        .route("/users/{id}", delete(user_handler::delete_user))


} 