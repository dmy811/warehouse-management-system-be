use std::collections::HashSet;

use std::env;

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
pub app_env: AppEnvironment,
    pub host: String,
    pub port: u16,
    pub rust_log: String,
    
    // Database
    pub database_url: String,
    pub database_pool_size: u32,
    pub database_timeout_seconds: u64,
    
    // Redis
    pub redis_urls: Vec<String>,
    pub redis_cluster_mode: bool,
    pub redis_pool_size: usize,
    pub redis_timeout_seconds: u64,
    pub use_redis_cache: bool,
    
    // PASETO for access token
    pub paseto_symmetric_key: Vec<u8>,
    
    // HMAC secret for refresh token hashing
    pub refresh_token_hmac_secret: Vec<u8>,
    
    // Token settings
    pub access_token_ttl_seconds: i64,
    pub refresh_token_ttl_days: i64,
    pub refresh_token_length_bytes: usize,
    
    // Cookie
    pub cookie_name: String,
    pub cookie_secure: bool,
    pub cookie_http_only: bool,
    pub cookie_same_site: SameSite,
    pub cookie_domain: String,
    pub cookie_path: String,
    pub cookie_max_age_seconds: i64,
    
    // CORS
    pub cors_allowed_origins: HashSet<String>,
    
    // Rate limiting
    pub rate_limit_per_ip: u32,
    pub rate_limit_window_seconds: u64,
    pub rate_limit_burst: u32,
    
    // Security
    pub csp_header: String,
    pub hsts_max_age: u64,
    
    // Audit
    pub audit_log_enabled: bool,
    pub failed_login_threshold: u32,
    pub failed_login_lockout_minutes: u64,
    
    // Metrics
    pub metrics_enabled: bool,
    pub metrics_path: String,
    pub cloudinary: CloudinaryConfig
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppEnvironment {
    Development,
    Staging,
    Production,
}

#[derive(Debug, Clone)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
app_env: match env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()).as_str() {
                "production" => AppEnvironment::Production,
                "staging" => AppEnvironment::Staging,
                _ => AppEnvironment::Development,
            },
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT").unwrap_or_else(|_| "3000".to_string()).parse()?,
            rust_log: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            
            database_url: env::var("DATABASE_URL")?,
            database_pool_size: env::var("DATABASE_POOL_SIZE").unwrap_or_else(|_| "50".to_string()).parse()?,
            database_timeout_seconds: env::var("DATABASE_TIMEOUT_SECONDS").unwrap_or_else(|_| "5".to_string()).parse()?,
            
            redis_urls: env::var("REDIS_URLS")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string())
                .split(',')
                .map(|s| s.to_string())
                .collect(),
            redis_cluster_mode: env::var("REDIS_CLUSTER_MODE").unwrap_or_else(|_| "false".to_string()).parse()?,
            redis_pool_size: env::var("REDIS_POOL_SIZE").unwrap_or_else(|_| "20".to_string()).parse()?,
            redis_timeout_seconds: env::var("REDIS_TIMEOUT_SECONDS").unwrap_or_else(|_| "2".to_string()).parse()?,
            use_redis_cache: env::var("USE_REDIS_CACHE").unwrap_or_else(|_| "true".to_string()).parse()?,
            
            paseto_symmetric_key: hex::decode(env::var("PASETO_SYMMETRIC_KEY")?)?,
            refresh_token_hmac_secret: hex::decode(env::var("REFRESH_TOKEN_HMAC_SECRET")?)?,
            
            access_token_ttl_seconds: env::var("ACCESS_TOKEN_TTL_SECONDS").unwrap_or_else(|_| "900".to_string()).parse()?,
            refresh_token_ttl_days: env::var("REFRESH_TOKEN_TTL_DAYS").unwrap_or_else(|_| "7".to_string()).parse()?,
            refresh_token_length_bytes: env::var("REFRESH_TOKEN_LENGTH_BYTES").unwrap_or_else(|_| "32".to_string()).parse()?,
            
            cookie_name: env::var("COOKIE_NAME").unwrap_or_else(|_| "__Secure_refresh_token".to_string()),
            cookie_secure: env::var("COOKIE_SECURE").unwrap_or_else(|_| "true".to_string()).parse()?,
            cookie_http_only: env::var("COOKIE_HTTP_ONLY").unwrap_or_else(|_| "true".to_string()).parse()?,
            cookie_same_site: match env::var("COOKIE_SAMESITE").unwrap_or_else(|_| "Strict".to_string()).as_str() {
                "Lax" => SameSite::Lax,
                "None" => SameSite::None,
                _ => SameSite::Strict,
            },
            cookie_domain: env::var("COOKIE_DOMAIN").unwrap_or_else(|_| "localhost".to_string()),
            cookie_path: env::var("COOKIE_PATH").unwrap_or_else(|_| "/api/v1/auth/refresh".to_string()),
            cookie_max_age_seconds: env::var("COOKIE_MAX_AGE_SECONDS").unwrap_or_else(|_| "604800".to_string()).parse()?,
            
            cors_allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "https://app.example.com".to_string())
                .split(',')
                .map(|s| s.to_string())
                .collect(),
            
            rate_limit_per_ip: env::var("RATE_LIMIT_PER_IP").unwrap_or_else(|_| "100".to_string()).parse()?,
            rate_limit_window_seconds: env::var("RATE_LIMIT_WINDOW_SECONDS").unwrap_or_else(|_| "60".to_string()).parse()?,
            rate_limit_burst: env::var("RATE_LIMIT_BURST").unwrap_or_else(|_| "150".to_string()).parse()?,
            
            csp_header: env::var("CSP_HEADER").unwrap_or_else(|_| "default-src 'self'".to_string()),
            hsts_max_age: env::var("HSTS_MAX_AGE").unwrap_or_else(|_| "31536000".to_string()).parse()?,
            
            audit_log_enabled: env::var("AUDIT_LOG_ENABLED").unwrap_or_else(|_| "true".to_string()).parse()?,
            failed_login_threshold: env::var("FAILED_LOGIN_THRESHOLD").unwrap_or_else(|_| "5".to_string()).parse()?,
            failed_login_lockout_minutes: env::var("FAILED_LOGIN_LOCKOUT_MINUTES").unwrap_or_else(|_| "15".to_string()).parse()?,
            
            metrics_enabled: env::var("METRICS_ENABLED").unwrap_or_else(|_| "true".to_string()).parse()?,
            metrics_path: env::var("METRICS_PATH").unwrap_or_else(|_| "/metrics".to_string()).parse()?,
            cloudinary: CloudinaryConfig::from_env()?
        })
    }

    pub fn is_production(&self) -> bool {
        self.app_env == AppEnvironment::Production
    }
}

#[derive(Debug, Clone)]
pub struct CloudinaryConfig {
    pub cloud_name: String,
    pub api_key: String,
    pub api_secret: String
}

impl CloudinaryConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            cloud_name: std::env::var("CLOUDINARY_CLOUD_NAME").context("CLOUDINARY_CLOUD_NAME must be set")?,
            api_key: std::env::var("CLOUDINARY_API_KEY").context("CLOUDINARY_API_KEY must be set")?,
            api_secret: std::env::var("CLOUDINARY_API_SECRET").context("CLOUDINARY_API_SECRET must be set")?
        })
    }
    pub fn upload_url(&self) -> String {
        format!("https://api.cloudinary.com/v1_1/{}/image/upload", self.cloud_name)
    }
}