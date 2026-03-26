use once_cell::sync::Lazy;
use regex::Regex;

pub mod roles {
    pub const ADMIN: &str = "ADMIN";
    pub const MANAGER: &str = "MANAGER";
    pub const STAFF: &str = "STAFF";
    pub const VIEWER: &str = "VIEWER";
}

pub mod pagination {
    pub const DEFAULT_PAGE: i64 = 1;
    pub const DEFAULT_PER_PAGE: i64 = 20;
    pub const MAX_PER_PAGE: i64 = 100;
}

pub mod file_upload {
    pub const MAX_FILE_SIZE: usize = 5 * 1024 * 1024; // 5mb
    pub const ALLOWED_MIME_TYPES: &[&str] = &["image/jpeg", "image/png", "image/webp"];
    pub const ALLOWED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp"];
}

pub mod stock {
    pub const LOW_STOCK_MULTIPLIER: f64 = 1.2;
}

pub mod headers {
    pub const REQUEST_ID: &str = "x-request-id";
    pub const REQUEST_ID_RESPONSE: &str = "x-request-id";
}

pub static PASSWORD_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d).+$").unwrap()
});

pub static PHONE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\+62|08)[0-9]{8,12}$").unwrap()
});