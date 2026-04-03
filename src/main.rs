use tokio::signal;
use warehouse_management_system_backend::infrastructure::config::Config;
use warehouse_management_system_backend::infrastructure::logger::init_logger;
use warehouse_management_system_backend::app;

use tracing::info;
 
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env
    dotenvy::dotenv().ok();
    
    // Init config (panics early if env vars missing)
    let config = Config::from_env()?;
 
    // Init tracing
    init_logger(&config);
 
    // Build app
    let app = app::build(config).await?;
 
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Server listening on {}", addr);
 
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await?;
 
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };
    
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}