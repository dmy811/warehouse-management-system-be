use anyhow::Result;
use axum::{Router, extract::DefaultBodyLimit, middleware};
use sqlx::PgPool;
use tower_http::{compression::CompressionLayer, cors::{Any, CorsLayer}, trace::TraceLayer};

use crate::{infrastructure::{config::Config, db::create_pool}, middlewares::{auth_middleware, logging_middleware, request_id_middleware}, routes::{auth_protected_routes, auth_public_routes, auth_routes, health_routes, warehouse_routes}, state::AppState};
use crate::constants::file_upload::MAX_FILE_SIZE;

pub async fn build(config: Config) -> Result<Router> {
    let pool = create_pool(&config.database_url).await?;
    Ok(build_with_pool(pool, config).await)
}

pub async fn build_with_pool(pool: PgPool, config: Config) -> Router {
    let state = AppState::new(pool, config);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let public = Router::new()
        .merge(auth_public_routes())
        .merge(health_routes());

    let protected = Router::new()
        .merge(auth_protected_routes())
        .merge(warehouse_routes())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware
        ));

    let api = Router::new()
        .merge(public)
        .merge(protected);
        // .merge(upload_routes());

    Router::new()
        .nest("/api/v1", api)
        .layer(DefaultBodyLimit::max(MAX_FILE_SIZE + 1024))
        .layer(middleware::from_fn(logging_middleware))
        .layer(middleware::from_fn(request_id_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(cors)
        .with_state(state)
}