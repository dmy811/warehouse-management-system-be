use axum::{Json, response::IntoResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default = "default_page")] // page number
    pub page: i64,

    #[serde(default = "default_per_page")] // items per page 
    pub per_page: i64,

    pub search: Option<String>, // search by name or address (case-insensitive, partial match)

    #[serde(default = "default_sort_by")] // sort field: name | created_at | updated_at
    pub sort_by: String,

    #[serde(default = "default_sort_order")] // sort direction: asc | desc
    pub sort_order: String,
}

fn default_page() -> i64 { 1 }
fn default_per_page() -> i64 { 20 }
fn default_sort_by() -> String { "created_at".to_string() }
fn default_sort_order() -> String { "desc".to_string() }

impl ListQuery {
    pub fn offset(&self) -> i64 { // atau skip
        (self.page.max(1) - 1) * self.per_page()
    }

    pub fn per_page(&self) -> i64 {
        self.per_page.clamp(1, 100)
    }

    pub fn sort_column(&self) -> &str {
        match self.sort_by.as_str() {
            "name" => "w.name",
            "updated_at" => "w.updated_at",
            _ => "w.created_at" // default
        }
    }

    pub fn sort_direction(&self) -> &str {
        match self.sort_order.to_lowercase().as_str() {
            "asc" => "ASC",
            _ => "DESC", // default
        }
    }
}


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