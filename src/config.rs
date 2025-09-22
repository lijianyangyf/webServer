use crate::error::AppError;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub server_address: String,
    pub database_url: String,
    pub rust_log: String,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        dotenvy::dotenv().ok();

        let server_address = env::var("SERVER_ADDRESS")
            .map_err(|_| AppError::Config("SERVER_ADDRESS must be set".to_string()))?;
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| AppError::Config("DATABASE_URL must be set".to_string()))?;
        let rust_log = env::var("RUST_LOG")
            .map_err(|_| AppError::Config("RUST_LOG must be set".to_string()))?;

        Ok(Self {
            server_address,
            database_url,
            rust_log,
        })
    }
}
