use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expires_in_secs: i64,
    pub app_env: AppEnv,
    pub cloudinary: CloudinaryConfig
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppEnv {
    Development,
    Production
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?,
            jwt_secret: std::env::var("JWT_SECRET").context("JWT_SECRET must be set")?,
            jwt_expires_in_secs: std::env::var("JWT_EXPIRES_IN_SECS")
                .unwrap_or_else(|_| "86400".to_string())
                .parse()
                .context("JWT_EXPIRES_IN_SECS must be a valid number")?,
            app_env: match std::env::var("APP_ENV")
                .unwrap_or_else(|_| "development".to_string())
                .as_str() {
                    "production" => AppEnv::Production,
                    _ => AppEnv::Development
                },
            cloudinary: CloudinaryConfig::from_env()?
        })
    }

    pub fn is_production(&self) -> bool {
        self.app_env == AppEnv::Production
    }

    pub fn dummy() -> Self {
        Self {
            database_url: String::new(),
            jwt_secret: String::new(),
            jwt_expires_in_secs: 0,
            app_env: AppEnv::Development,
            cloudinary: CloudinaryConfig::dummy(),
        }
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

    pub fn dummy() -> Self {
        Self {
            cloud_name: String::new(),
            api_key: String::new(),
            api_secret: String::new()
        }
    }
}