use anyhow::Result;
use axum::{Router, extract::DefaultBodyLimit, middleware};
use sqlx::PgPool;
use deadpool_redis::Pool as RedisPool;
use tower_http::{compression::CompressionLayer, cors::{Any, CorsLayer, AllowOrigin, AllowMethods, AllowHeaders}, trace::TraceLayer};

use crate::{infrastructure::{config::Config, db::create_pool, redis}, middlewares::{auth_middleware, logging_middleware, request_id_middleware}, routes::{auth_protected_routes, auth_public_routes, health_routes, role_routes::role_routes, user_role_routes::user_role_routes, user_routes::user_routes, user_warehouse_routes::user_warehouse_routes, warehouse_routes}, state::AppState};
use crate::constants::file_upload::MAX_FILE_SIZE;

pub async fn build(config: Config) -> Result<Router> {
    let pg_pool = create_pool(&config.database.url).await?;
    let redis_pool = redis::create_pool(&config.redis).await?;
    Ok(build_with_pool(pg_pool, redis_pool, config).await)
}

pub async fn build_with_pool(pg_pool: PgPool, redis_pool: RedisPool, config: Config) -> Router {
    let cors = build_cors(&config);
    let state = AppState::new(pg_pool, redis_pool, config);

    let public = Router::new()
        .merge(auth_public_routes())
        .merge(health_routes());

    let protected = Router::new()
        .merge(auth_protected_routes())
        .merge(user_routes())
        .merge(role_routes())
        .merge(user_role_routes())
        .merge(warehouse_routes())
        .merge(user_warehouse_routes())
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

fn build_cors(config: &Config) -> CorsLayer {
    let origins = &config.cors.allowed_origins;

    if origins.is_empty() { // No origins configured, allow none in production, any in dev
        if config.is_production(){
            return CorsLayer::new()
        } else {
            return CorsLayer::permissive();
            // allow
            // semua origin
            // semua method
            // semua header
            // shortcut utk:
            // CorsLayer::new()
            //     .allow_origin(Any)
            //     .allow_methods(Any)
            //     .allow_headers(Any)
        }
    }

    let allow_origin: Vec<axum::http::HeaderValue> = origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();

    let mut layer = CorsLayer::new()
        .allow_origin(AllowOrigin::list(allow_origin))
        .allow_methods(AllowMethods::any())
        .allow_headers(AllowHeaders::any());
 
    if config.cors.allow_credentials {
        layer = layer.allow_credentials(true);
    }
 
    layer
}