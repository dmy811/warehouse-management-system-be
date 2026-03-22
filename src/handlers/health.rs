// use axum::{Json, extract::State};
// use serde::Serialize;

// use crate::{state::AppState, utils::response::{success_response, success_response_with_message}};

// #[derive(Serialize)]
// pub struct DataHealthResponse {
//     pub timestamp: String,
//     pub database: String
// }

// pub async fn health_check(State(state): State<AppState>) -> (axum::http::StatusCode, Json<crate::utils::response::ApiResponse<DataHealthResponse>>) {
//     let db_status = match sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(&state.pool).await {
//         Ok(_) => "connected".to_string(),
//         Err(_) => "disconnected".to_string()
//     };

//     let data_response = DataHealthResponse {
//         timestamp: chrono::Utc::now().to_rfc3339(),
//         database: db_status
//     };
    
//     success_response_with_message(data_response, "health check successful".to_string())
// }