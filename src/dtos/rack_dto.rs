use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::models::{Rack, RackWithStats};

// use serde::Deserialize;
// use validator::Validate;
// use regex::Regex;
// use std::sync::LazyLock; // Gunakan once_cell::sync::Lazy jika menggunakan Rust versi < 1.80

// // Contoh: Regex untuk memastikan kode rak hanya berisi huruf kapital, angka, dan strip (misal: "RACK-A01")
// static RACK_CODE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Z0-9\-]+$").unwrap());

// #[derive(Debug, Deserialize, Validate)]
// pub struct CreateRackRequest {
//     // ID harus berupa angka positif (tidak mungkin 0 atau negatif di database relasional pada umumnya)
//     #[validate(range(min = 1, message = "Warehouse ID tidak valid"))]
//     pub warehouse_id: i64,

//     // Kode harus memiliki panjang yang wajar dan format yang seragam untuk memudahkan pencarian
//     #[validate(
//         length(min = 2, max = 50, message = "Code harus terdiri dari 2 hingga 50 karakter"),
//         regex(path = *RACK_CODE_REGEX, message = "Code hanya boleh berisi huruf kapital, angka, dan tanda strip")
//     )]
//     pub code: String,

//     // Option akan otomatis diabaikan jika bernilai None. 
//     // Tapi jika diisi (Some), validasi ini akan berjalan.
//     #[validate(length(min = 1, max = 20, message = "Zone tidak boleh kosong jika diisi, maksimal 20 karakter"))]
//     pub zone: Option<String>,

//     // Level rak biasanya mulai dari lantai 1 ke atas. Kita beri batas wajar maksimal 100.
//     #[validate(range(min = 1, max = 100, message = "Level rak harus berada di antara 1 hingga 100"))]
//     pub level: Option<i32>,

//     // Kapasitas rak tidak boleh negatif.
//     #[validate(range(min = 0, message = "Kapasitas tidak boleh bernilai negatif"))]
//     pub capacity: Option<i64>,

//     // Deskripsi dibatasi agar user tidak mengirim payload teks yang terlalu masif (mencegah DoS/Memory Bloat)
//     #[validate(length(max = 255, message = "Deskripsi tidak boleh lebih dari 255 karakter"))]
//     pub description: Option<String>
// }

// #[derive(Debug, Deserialize, Validate)]
// pub struct CreateRackRequest {
//     #[validate(range(min = 1, message = "Warehouse ID not valid"))]
//     pub warehouse_id: i64,

//     #[validate(
//         length(min = 2, max = 20, message = "Code must be between 2 and 20 characters"),
//         regex(path = )
//     )]
//     pub code: String,

//     #[validate(length(min = 1, max = 10, message = "Zone must be between 1 and 10 characters"))]
//     pub zone: Option<String>,


//     pub level: Option<i32>,
//     pub capacity: Option<i64>,
//     pub description: Option<String>
// }

#[derive(Debug, Serialize)]
pub struct RackResponse {
    pub id: i64,
    pub warehouse_id: i64,
    pub code: String,
    pub zone: Option<String>,       
    pub level: Option<i32>,          
    pub description: Option<String>,
    pub capacity: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

impl From<Rack> for RackResponse {
    fn from(r: Rack) -> Self {
        Self {
            id: r.id,
            warehouse_id: r.warehouse_id,
            code: r.code,
            zone: r.zone,
            level: r.level,
            description: r.description,
            capacity: r.capacity,
            created_at: r.created_at,
            updated_at: r.updated_at
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RackSummary {
    pub id: i64,
    pub warehouse_id: i64,
    pub code: String,
    pub zone: Option<String>,       
    pub level: Option<i32>,          
    pub description: Option<String>,
    pub capacity: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub used_capacity: i64,
    pub total_products: i64,
}

impl From<RackWithStats> for RackSummary {
    fn from(r: RackWithStats) -> Self {
        Self {
            id: r.id,
            warehouse_id: r.warehouse_id,
            code: r.code,
            zone: r.zone,
            level: r.level,
            description: r.description,
            capacity: r.capacity,
            created_at: r.created_at,
            updated_at: r.updated_at,
            used_capacity: r.used_capacity,
            total_products: r.total_products
        }
    }
}