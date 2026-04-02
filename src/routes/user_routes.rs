use axum::{Router, routing::{patch, post, get}};

use crate::{handlers::user_handler, state::AppState};

pub fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/users", get(user_handler::list_all_users))
        .route("/users", post(user_handler::create_user))
        .route("/users/{id}", patch(user_handler::update_user))
} 