use axum::{Router, middleware, routing::{get, post}};

use crate::{handlers::auth_handler, state::AppState, middlewares::auth_middleware};

pub fn auth_routes() -> Router<AppState> {
    let public = Router::new()
        .route("/auth/register", post(auth_handler::register))
        .route("/auth/login", post(auth_handler::login));

// AppState::dummy() satisfies the type system at build time;
// Axum replaces it with the real state via .with_state() in app.rs.

    let protected = Router::new()
        .route("/auth/me", get(auth_handler::me))
        .route_layer(middleware::from_fn_with_state(
            AppState::dummy(),
            auth_middleware
        ));

    Router::new().merge(public).merge(protected)
}