use axum::{extract::State, response::IntoResponse};
use serde_json::json;
use tracing::info;

use crate::{errors::AppResult, response::ApiResponse, state::AppState};

pub async fn ping() -> impl IntoResponse {
    ApiResponse::ok("pong", serde_json::Value::Null)
}

pub async fn db_health_check(State(state): State<AppState>) -> AppResult<impl IntoResponse> {
    let db_ok = sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .is_ok();

    let pool_size = state.db.size(); // check active connection
    let pool_idle = state.db.num_idle(); // idle connection

    let status = if db_ok { "healthy" } else { "unhealthy" };

    let data = json!({
        "status": status,
        "checks": {
            "database": if db_ok { "ok" } else { "error" }
        },
        "pool": {
            "size": pool_size,
            "idle": pool_idle
        }
    });

    if!db_ok {
        return Err(crate::errors::AppError::ServiceUnavailable(
            "Database connection failed".to_string()
        ));
    }

    Ok(ApiResponse::ok("Service is healthy", data))
}