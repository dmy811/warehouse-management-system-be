use anyhow::Result;
use axum::{Router, extract::DefaultBodyLimit, middleware};
use sqlx::PgPool;
use tower_http::{compression::CompressionLayer, cors::{Any, CorsLayer}, trace::TraceLayer};

use crate::{infrastructure::{config::Config, db::create_pool}, middlewares::{logging_middleware, request_id_middleware}, routes::{auth_routes, health_routes}, state::AppState};
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

    let api = Router::new()
        .merge(health_routes())
        .merge(auth_routes());
        // .merge(warehouse_routes())
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