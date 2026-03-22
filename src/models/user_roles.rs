use serde::Serialize;

#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct UserRole {
    pub id: i64,
    pub user_id: i64,
    pub role_id: i64,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>
}

pub const DEFAULT_ROLE: &str = "keeper";