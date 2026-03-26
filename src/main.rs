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
mod handlers;

use infrastructure::config::Config;
use tracing::info;
 
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env
    dotenvy::dotenv().ok();
 
    // Init config (panics early if env vars missing)
    let config = Config::from_env()?;
 
    // Init tracing
    infrastructure::logger::init_logger(&config);
 
    // Build app
    let app = app::build(config).await?;
 
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Server listening on {}", addr);
 
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
 
    Ok(())
}