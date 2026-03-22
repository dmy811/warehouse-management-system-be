use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::infrastructure::config::{AppEnv, Config};

pub fn init(config: &Config) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn,tower_http=debug"));

    match config.app_env {
        AppEnv::Production => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer().json())
                .init()
        }
        AppEnv::Development => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer().pretty())
                .init()
        }
    }
}