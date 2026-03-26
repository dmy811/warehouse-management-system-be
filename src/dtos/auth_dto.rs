use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::models::UserWithRole;
use crate::constants::PASSWORD_REGEX;
use crate::constants::PHONE_REGEX;

// --- Request DTOs ---
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 2, max = 100, message = "Name must be between 2 and 100 characters!"))]
    pub name: String,

    #[validate(email(message = "Invalid email address!"))]
    pub email: String,

    #[validate(length(min = 8, max = 20, message = "Phone must be between 8 and 20 characters!"), regex(path = "*PHONE_REGEX", message = "Invalid phone number format"))]
    pub phone: Option<String>,
    
    #[validate(length(min = 8, message = "Password must at least 8 and 20 characters!"), regex(path = "*PASSWORD_REGEX", message = "Password must contain uppercase, lowercase, and number"))]
    pub password: String,

    #[validate(length(min = 8, message = "Password must at least 8 and 20 characters!"), must_match(other = "password", message = "Password and Confirm Password doesn't match!"))]
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
pub struct UserResponse {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub photo: Option<String>,
    pub role: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

impl From<UserWithRole> for UserResponse {
    fn from(u: UserWithRole) -> Self {
        Self {
            id: u.id,
            name: u.name,
            email: u.email,
            phone: u.phone,
            photo: u.photo,
            role: u.role_name,
            created_at: u.created_at
        }
    }
}

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