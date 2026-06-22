use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::dtos::UserResponse;
use crate::models::UserWithRole;
use crate::validators::common::{validate_indonesian_phone, validate_password_strength};

// --- Request DTOs ---
#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePasswordRequest {
    #[validate(custom(function = "validate_password_strength"))]
    pub password: String,

    #[validate(length(min = 8, message = "Password must at least 8"), must_match(other = "password", message = "Password and Confirm Password doesn't match!"))]
    pub password_confirm: String
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email address!"))]
    pub email: String,

    #[validate(length(min = 1, message = "Password is required!"))]
    pub password: String,
}

// --- Response DTOs ---
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub user: UserResponse
}

impl AuthResponse {
    pub fn new(token: String, user: UserWithRole) -> Self {
        Self {
            access_token: token,
            token_type: "Bearer".to_string(),
            user: UserResponse::from(user)
        }
    }
}