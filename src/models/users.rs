use serde::Serialize;

#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub password: String,
    pub photo: Option<String>,
    pub phone: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>
}
