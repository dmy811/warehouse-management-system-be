use axum::{Router, routing::{patch, post}};

use crate::{handlers::user_handler, state::AppState};

pub fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/users", post(user_handler::create))
        .route("/users/{id}", patch(user_handler::update))
} 