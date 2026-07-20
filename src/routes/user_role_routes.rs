use axum::{Router, routing::{post}};

use crate::{handlers::user_role_handler, state::AppState};

pub fn user_role_routes() -> Router<AppState> {
    Router::new()
        .route("/users/{user_id}/roles", post(user_role_handler::assign_role_to_user))
}