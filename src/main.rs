mod app;
mod constants;
mod dtos;
mod errors;
mod infrastructure;
mod middlewares;
mod models;
mod repositories;
mod response;
mod routes;
mod services;
mod state;
mod utils;

// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//     dotenvy::dotenv().ok();

//     tracing_subscriber::fmt()
//         .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
//         .init();

//     let pool = infrastructure::db::init_db_pool().await?;
//     let jwt = infrastructure::security::JwtConfig::from_env()?;
//     let state = state::AppState {pool, jwt};

//     let app = routes::build_router(state);


//     let raw_host = std::env::var("SERVER_HOST").ok();
//     let raw_app_port = std::env::var("SERVER_PORT").ok();

//     let host = raw_host.clone().unwrap_or_else(|| "0.0.0.0".to_string());
//     let port: u16 = raw_app_port
//         .clone()
//         .and_then(|s| s.parse().ok())
//         .unwrap_or(3000);

//     tracing::debug!(host=?raw_host, app_port=?raw_app_port, "Resolved server binding env values");
//     let addr_str = format!("{}:{}", host, port);
//     let listener = tokio::net::TcpListener::bind(&addr_str).await?;
//     tracing::info!("Server running on {}", addr_str);
//     axum::serve(listener, app).await?;

//     Ok(())
// }
fn main() {
    println!("Hello, world!");
}