use axum::{Json, http::StatusCode, response::{IntoResponse, Response}};
use serde::Serialize;
use serde_json::{Value, json};


pub struct ApiResponse {
    status: StatusCode,
    message: &'static str,
    data: Value
}

impl ApiResponse {
    pub fn ok<T: Serialize>(message: &'static str, data: T) -> Self {
        Self {
            status: StatusCode::OK,
            message,
            data: serde_json::to_value(data).unwrap_or(Value::Null)
        }
    }

    pub fn created<T: Serialize>(message: &'static str, data: T) -> Self {
        Self {
            status: StatusCode::CREATED,
            message,
            data: serde_json::to_value(data).unwrap_or(Value::Null)
        }
    }

    pub fn no_content() -> impl IntoResponse {
        StatusCode::NO_CONTENT
    }
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> Response{
        let body = json!({
            "success": true,
            "message": self.message,
            "data": self.data
        });

        (self.status, Json(body)).into_response()
    }
}