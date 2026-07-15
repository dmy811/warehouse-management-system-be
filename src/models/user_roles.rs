use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct UserRoles {
    pub id: i64,
    pub user_id: i64,
    pub role_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}