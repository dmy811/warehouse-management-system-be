use serde::Deserialize;
use validator::Validate;

use crate::{errors::{AppError}, validators::common::{validate_indonesian_phone, validate_password_strength, validate_role}};

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 2, max = 100, message = "Name must be between 2 and 100 characters!"))]
    pub name: String,

    #[validate(email(message = "Invalid email address!"))]
    pub email: String,

    // #[validate(length(min = 10, max = 15, message = "Phone must be between 10 and 15 characters!"), regex(path = "*PHONE_REGEX", message = "Invalid phone number format"))]
    #[validate(custom(function = "validate_indonesian_phone"))]
    pub phone: Option<String>,

    #[validate(custom(function = "validate_role"))]
    pub role: String,
    
    // #[validate(length(min = 8, message = "Password must at least 8"), regex(path = "*PASSWORD_REGEX", message = "Password must contain uppercase, lowercase, and number"))]
    #[validate(custom(function = "validate_password_strength"))]
    pub password: String,

    #[validate(length(min = 8, message = "Password must at least 8"), must_match(other = "password", message = "Password and Confirm Password doesn't match!"))]
    pub password_confirm: String
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(min = 2, max = 100, message = "Name must be between 2 and 100 characters!"))]
    pub name: Option<String>,

    #[validate(email(message = "Invalid email address!"))]
    pub email: Option<String>,

    #[validate(custom(function = "validate_indonesian_phone"))]
    pub phone: Option<String>,
}

impl UpdateUserRequest {
    pub fn validate_is_empty(&self) -> Result<(), AppError> {
        if self.name.is_none() && self.email.is_none() && self.phone.is_none() {
            return Err(AppError::Validation(
                "At least one field must be provided".to_string()
            ));
        }

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct DeleteUserQuery {
    pub mode: Option<String>
}