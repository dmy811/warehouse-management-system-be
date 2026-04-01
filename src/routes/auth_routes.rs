use axum::{Router, routing::{delete, get, patch, post, put}};

use crate::{handlers::{auth_handler, upload_handler}, state::AppState};

pub fn auth_public_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(auth_handler::login))
}

pub fn auth_protected_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/me", get(auth_handler::get_profile))
        .route("/auth/me", patch(auth_handler::update_profile))
        .route("/auth/me/password", put(auth_handler::change_profile_password))
        .route("/auth/me/photo", post(upload_handler::upload_user_photo))
        .route("/auth/me/photo", delete(upload_handler::delete_user_photo))
}