use serde::Serialize;

#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct Role {
    pub id: i64,
    pub name: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>
}