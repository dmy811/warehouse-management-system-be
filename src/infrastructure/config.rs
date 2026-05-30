use anyhow::{Context, Result};
use std::time::Duration;

// --- App ----------------------------------

#[derive(Debug, Clone)]
pub struct Config {
    pub app: AppConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub auth: AuthConfig,
    pub cookie: CookieConfig,
    pub cors: CorsConfig,
    pub rate_limit: RateLimitConfig,
    pub cloudinary: CloudinaryConfig,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            app: AppConfig::from_env()?,
            database: DatabaseConfig::from_env()?,
            redis: RedisConfig::from_env()?,
            auth: AuthConfig::from_env()?,
            cookie: CookieConfig::from_env()?,
            cors: CorsConfig::from_env()?,
            rate_limit: RateLimitConfig::from_env()?,
            cloudinary: CloudinaryConfig::from_env()?,
        })
    }

    pub fn is_production(&self) -> bool {
        self.app.env == AppEnv::Production
    }
}

// --- AppConfig ----------------------------------

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub env: AppEnv,
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

impl AppConfig {
    fn from_env() -> Result<Self> {
        Ok(Self {
            env: match std::env::var("APP_ENV")
                .unwrap_or_else(|_| "development".into())
                .as_str()
            {
                "production" => AppEnv::Production,
                _ => AppEnv::Development,
            },
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8080".into())
                .parse()
                .context("PORT must be a valid number")?,
            workers: std::env::var("WORKERS")
                .unwrap_or_else(|_| "4".into())
                .parse()
                .context("WORKERS must be a valid number")?,
        })
    }

    pub fn socket_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppEnv {
    Development,
    Production,
}

// --- DatabaseConfig ----------------------------------

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
    pub timeout: Duration,
}

impl DatabaseConfig {
    fn from_env() -> Result<Self> {
        Ok(Self {
            url: std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?,
            pool_size: std::env::var("DATABASE_POOL_SIZE")
                .unwrap_or_else(|_| "10".into())
                .parse()
                .context("DATABASE_POOL_SIZE must be a number")?,
            timeout: Duration::from_secs(
                std::env::var("DATABASE_TIMEOUT_SECONDS")
                    .unwrap_or_else(|_| "5".into())
                    .parse()
                    .context("DATABASE_TIMEOUT_SECONDS must be a number")?,
            ),
        })
    }
}

// --- RedisConfig ----------------------------------

#[derive(Debug, Clone)]
pub struct RedisConfig {
    /// Comma-separated Redis URLs. Single URL for standalone, multiple for cluster.
    pub url: String,
    pub pool_size: usize,
    pub timeout: Duration,
    pub use_cache: bool,
}

impl RedisConfig {
    fn from_env() -> Result<Self> {
        Ok(Self {
            url: std::env::var("REDIS_URL").unwrap_or_default(),
            pool_size: std::env::var("REDIS_POOL_SIZE")
                .unwrap_or_else(|_| "10".into())
                .parse()
                .context("REDIS_POOL_SIZE must be a number")?,
            timeout: Duration::from_secs(
                std::env::var("REDIS_TIMEOUT_SECONDS")
                    .unwrap_or_else(|_| "2".into())
                    .parse()
                    .context("REDIS_TIMEOUT_SECONDS must be a number")?,
            ),
            use_cache: std::env::var("USE_REDIS_CACHE")
                .unwrap_or_else(|_| "false".into())
                .parse()
                .unwrap_or(false),
        })
    }
}

// --- AuthConfig ----------------------------------

#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// PASETO v4 local symmetric key — must be exactly 32 bytes, base64-encoded
    pub paseto_key: String,
    /// HMAC secret for signing refresh tokens
    pub refresh_token_hmac_secret: String,
    /// Refresh token TTL in days
    pub refresh_token_ttl_days: u64,
    /// Number of random bytes for refresh token generation
    pub refresh_token_length_bytes: usize,
    /// Failed login threshold before lockout
    pub failed_login_threshold: u32,
    /// Lockout duration in minutes
    pub failed_login_lockout_minutes: u64,
}

impl AuthConfig {
    fn from_env() -> Result<Self> {
        Ok(Self {
            paseto_key: std::env::var("PASETO_SYMMETRIC_KEY")
                .context("PASETO_SYMMETRIC_KEY must be set")?,
            refresh_token_hmac_secret: std::env::var("REFRESH_TOKEN_HMAC_SECRET")
                .context("REFRESH_TOKEN_HMAC_SECRET must be set")?,
            refresh_token_ttl_days: std::env::var("REFRESH_TOKEN_TTL_DAYS")
                .unwrap_or_else(|_| "7".into())
                .parse()
                .context("REFRESH_TOKEN_TTL_DAYS must be a number")?,
            refresh_token_length_bytes: std::env::var("REFRESH_TOKEN_LENGTH_BYTES")
                .unwrap_or_else(|_| "32".into())
                .parse()
                .context("REFRESH_TOKEN_LENGTH_BYTES must be a number")?,
            failed_login_threshold: std::env::var("FAILED_LOGIN_THRESHOLD")
                .unwrap_or_else(|_| "5".into())
                .parse()
                .context("FAILED_LOGIN_THRESHOLD must be a number")?,
            failed_login_lockout_minutes: std::env::var("FAILED_LOGIN_LOCKOUT_MINUTES")
                .unwrap_or_else(|_| "15".into())
                .parse()
                .context("FAILED_LOGIN_LOCKOUT_MINUTES must be a number")?,
        })
    }

    pub fn refresh_token_ttl_seconds(&self) -> u64 {
        self.refresh_token_ttl_days * 24 * 60 * 60
    }

    pub fn lockout_seconds(&self) -> u64 {
        self.failed_login_lockout_minutes * 60
    }
}

// --- CookieConfig ----------------------------------

#[derive(Debug, Clone)]
pub struct CookieConfig {
    pub name: String,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: String,
    pub domain: Option<String>,
    pub path: String,
    pub max_age_seconds: i64,
}

impl CookieConfig {
    fn from_env() -> Result<Self> {
        Ok(Self {
            name: std::env::var("COOKIE_NAME")
                .unwrap_or_else(|_| "refresh_token".into())
                .trim_matches('"')
                .to_string(),
            secure: std::env::var("COOKIE_SECURE")
                .unwrap_or_else(|_| "true".into())
                .parse()
                .unwrap_or(true),
            http_only: std::env::var("COOKIE_HTTP_ONLY")
                .unwrap_or_else(|_| "true".into())
                .parse()
                .unwrap_or(true),
            same_site: std::env::var("COOKIE_SAME_SITE")
                .unwrap_or_else(|_| "Strict".into())
                .trim_matches('"')
                .to_string(),
            domain: std::env::var("COOKIE_DOMAIN").ok().map(|d| {
                d.trim_matches('"').to_string()
            }),
            path: std::env::var("COOKIE_PATH")
                .unwrap_or_else(|_| "/api/v1/auth/refresh".into())
                .trim_matches('"')
                .to_string(),
            max_age_seconds: std::env::var("COOKIE_MAX_AGE_SECONDS")
                .unwrap_or_else(|_| "604800".into())
                .parse()
                .context("COOKIE_MAX_AGE_SECONDS must be a number")?,
        })
    }
}

// --- CorsConfig ----------------------------------

#[derive(Debug, Clone)]
pub struct CorsConfig {
    /// Comma-separated list of allowed origins
    pub allowed_origins: Vec<String>,
    pub allow_credentials: bool,
}

impl CorsConfig {
    fn from_env() -> Result<Self> {
        let raw = std::env::var("CORS_ALLOWED_ORIGINS").unwrap_or_default();
        let origins: Vec<String> = raw
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(Self {
            allowed_origins: origins,
            allow_credentials: std::env::var("CORS_ALLOW_CREDENTIALS")
                .unwrap_or_else(|_| "false".into())
                .parse()
                .unwrap_or(false),
        })
    }
}

// --- RateLimitConfig ----------------------------------

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Max requests per window per IP
    pub per_ip: u32,
    /// Window size in seconds
    pub window_seconds: u64,
    /// Burst allowance above the base limit
    pub burst: u32,
}

impl RateLimitConfig {
    fn from_env() -> Result<Self> {
        Ok(Self {
            per_ip: std::env::var("RATE_LIMIT_PER_IP")
                .unwrap_or_else(|_| "100".into())
                .parse()
                .context("RATE_LIMIT_PER_IP must be a number")?,
            window_seconds: std::env::var("RATE_LIMIT_WINDOW_SECONDS")
                .unwrap_or_else(|_| "60".into())
                .parse()
                .context("RATE_LIMIT_WINDOW_SECONDS must be a number")?,
            burst: std::env::var("RATE_LIMIT_BURST")
                .unwrap_or_else(|_| "150".into())
                .parse()
                .context("RATE_LIMIT_BURST must be a number")?,
        })
    }
}

// --- CloudinaryConfig ----------------------------------

#[derive(Debug, Clone)]
pub struct CloudinaryConfig {
    pub cloud_name: String,
    pub api_key: String,
    pub api_secret: String,
}

impl CloudinaryConfig {
    fn from_env() -> Result<Self> {
        Ok(Self {
            cloud_name: std::env::var("CLOUDINARY_CLOUD_NAME")
                .context("CLOUDINARY_CLOUD_NAME must be set")?,
            api_key: std::env::var("CLOUDINARY_API_KEY")
                .context("CLOUDINARY_API_KEY must be set")?,
            api_secret: std::env::var("CLOUDINARY_API_SECRET")
                .context("CLOUDINARY_API_SECRET must be set")?,
        })
    }

    pub fn upload_url(&self) -> String {
        format!(
            "https://api.cloudinary.com/v1_1/{}/image/upload",
            self.cloud_name
        )
    }
}