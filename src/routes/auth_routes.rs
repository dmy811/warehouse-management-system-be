use axum::{Router, middleware, routing::{get, post, delete}};

use crate::{handlers::{auth_handler, upload_handler}, state::AppState};

pub fn auth_public_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(auth_handler::register))
        .route("/auth/login", post(auth_handler::login))
}

pub fn auth_protected_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/me", get(auth_handler::me))
        .route("/auth/me/photo", post(upload_handler::upload_user_photo))
        .route("/auth/me/photo", delete(upload_handler::delete_user_photo))
}