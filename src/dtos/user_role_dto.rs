use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct AssignRoleRequest {
    #[validate(range(min = 1, message = "Role ID not valid"))]
    pub role_id: i64
}