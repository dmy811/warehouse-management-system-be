use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub password: String,
    pub photo: Option<String>,
    pub phone: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>
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
    pub role_name: Option<String>, // harus option karena left join
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}