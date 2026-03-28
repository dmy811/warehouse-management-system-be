use thiserror::Error;
use axum::{Json, http::StatusCode, response::{IntoResponse, Response}};
use tracing::error;
use serde_json::json;

use super::error_codes as code;

#[derive(Debug, Error)]
pub enum AppError {
    // auth
    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),

    #[error("Token is invalid or expired")]
    InvalidToken,

    #[error("You are not authorized to perform this action")]
    Forbidden,

    #[error("Authentication required")]
    Unauthorized,

    // domain
    #[error("{0} not found")]
    NotFound(String),

    #[error("{0} already exists")]
    Conflict(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Insufficient stock for product {product_id}: available {available}, requested {requested}")]
    InsufficientStock {
        product_id: i32,
        available: i32,
        requested: i32
    },

    // infrastructure
    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String)
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidCredentials(_) | Self::InvalidToken => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::InsufficientStock { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            Self::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::Database(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_code(&self) -> &'static str {
        match self {
            Self::InvalidCredentials(_) => code::INVALID_CREDENTIALS,
            Self::InvalidToken => code::INVALID_TOKEN,
            Self::Forbidden => code::FORBIDDEN,
            Self::Unauthorized => code::UNAUTHORIZED,
            Self::NotFound(_) => code::NOT_FOUND,
            Self::Conflict(_) => code::CONFLICT,
            Self::Validation(_) => code::VALIDATION_ERROR,
            Self::InsufficientStock { .. } => code::INSUFFICIENT_STOCK,
            Self::Database(_) => code::DATABASE_ERROR,
            Self::Internal(_) => code::INTERNAL_SERVER_ERROR,
            Self::ServiceUnavailable(_) => code::SERVICE_UNAVAILABLE
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();

        if status.is_server_error() {
            error!(error = %self, "Internal server error");
        }

        let message = match &self {
            Self::Database(_) | Self::Internal(_) => "An internal error ocurred".to_string(),
            other => other.to_string()
        };

        let body = json!({
            "success": false,
            "error": {
                "code": self.error_code(),
                "message": message
            }
        });

        (status, Json(body)).into_response()

    }
}

pub type AppResult<T> = Result<T, AppError>;