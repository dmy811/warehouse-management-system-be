use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Role {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>
}