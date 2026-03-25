use axum::{Json, response::IntoResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub items: Vec<T>,
    pub meta: PaginationMeta
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64
}

impl <T: Serialize> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: i64, page: i64, per_page: i64) -> Self {
        let total_pages = (total as f64 / per_page as f64).ceil() as i64;
        Self {
            items,
            meta: PaginationMeta {
                total,
                page,
                per_page,
                total_pages
            }
        }
    }
}

impl<T: Serialize> IntoResponse for PaginatedResponse<T> {
    fn into_response(self) -> axum::response::Response {
        let body = json!({
            "success": true,
            "data": self.items,
            "meta": self.meta,
        });
        Json(body).into_response()
    }
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: i64,

    #[serde(default = "default_per_page")]
    pub per_page: i64
}

impl PaginationQuery {
    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.per_page
    }
}

fn default_page() -> i64 {
    1
}

fn default_per_page() -> i64 {
    20
}