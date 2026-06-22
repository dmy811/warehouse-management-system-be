use chrono::{DateTime, Utc};
use serde::{Serialize};
use sqlx::FromRow;
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

// use chrono::{DateTime, Utc, TimeZone};

// // Chrono memiliki banyak method utility
// let now: DateTime<Utc> = Utc::now();
// let timestamp = now.timestamp();  // i64 seconds
// let timestamp_ms = now.timestamp_millis();
// let formatted = now.format("%Y-%m-%d %H:%M:%S").to_string();
// let parsed = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap();

// use time::{OffsetDateTime, format_description::well_known::Rfc3339};

// // Time lebih strict dan explicit
// let now: OffsetDateTime = OffsetDateTime::now_utc();
// let timestamp = now.unix_timestamp();  // i64 seconds
// let timestamp_ms = now.unix_timestamp_nanos() / 1_000_000;
// let formatted = now.format(&Rfc3339).unwrap();
// let parsed = OffsetDateTime::parse("2024-01-01T00:00:00Z", &Rfc3339).unwrap();
// let now = OffsetDateTime::now_utc();
// let expires_at = now + Duration::days(7);
// let timestamp = now.unix_timestamp();

// // Chrono - lebih banyak utility built-in
// use chrono::{Duration, Utc};

// let now = Utc::now();
// let later = now + Duration::minutes(15);
// let diff = later - now;  // Duration
// let days = diff.num_days();  // Method utility

// // Time - perlu explicit
// use time::{Duration, OffsetDateTime};

// let now = OffsetDateTime::now_utc();
// let later = now + Duration::minutes(15);
// let diff = later - now;  // Duration
// let days = diff.whole_days();  // Method juga ada

// // Jika terpaksa perlu konversi
// use chrono::{DateTime, Utc, TimeZone};
// use time::OffsetDateTime;

// // Chrono → Time
// fn chrono_to_time(dt: DateTime<Utc>) -> OffsetDateTime {
//     OffsetDateTime::from_unix_timestamp(dt.timestamp())
//         .unwrap()
//         .replace_nanosecond(dt.timestamp_subsec_nanos())
//         .unwrap()
// }

// // Time → Chrono
// fn time_to_chrono(dt: OffsetDateTime) -> DateTime<Utc> {
//     Utc.timestamp_opt(dt.unix_timestamp(), dt.nanosecond())
//         .unwrap()
// }