use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub password: String,
    pub photo: Option<String>,
    pub phone: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

// --- Roles model ---
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Role {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

// -- Joined result: users + their role name
#[derive(Debug, Clone, FromRow)]
pub struct UserWithRole {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub password: String,
    pub photo: Option<String>,
    pub phone: Option<String>,
    pub roles: Option<Vec<String>>, // harus option karena left join
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

#[derive(Debug, Clone, FromRow)]
pub struct RefreshToken {
    pub id: i64,
    pub token_id: Uuid,
    pub token_hash: String,
    pub user_id: i64,
    pub expires_at: OffsetDateTime,
    pub revoked_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub last_used_at: Option<OffsetDateTime>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}