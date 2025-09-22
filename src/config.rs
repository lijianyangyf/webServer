use crate::error::AppError;
use std::env;

/// 应用配置结构体，存储从环境变量加载的配置项。
#[derive(Debug, Clone)]
pub struct Config {
    /// 服务器监听地址，例如 "127.0.0.1:3000"。
    pub server_address: String,
    /// 数据库连接字符串。
    pub database_url: String,
    /// 日志级别，例如 "info", "debug"。
    pub rust_log: String,
}

impl Config {
    /// 从环境变量中加载配置。
    ///
    /// 这个函数会：
    /// 1. 使用 `dotenvy::dotenv().ok()` 尝试从项目根目录的 `.env` 文件加载环境变量。
    ///    这在本地开发时非常有用。如果 `.env` 文件不存在，此操作会被安全地忽略。
    /// 2. 逐一读取必要的环境变量 (`SERVER_ADDRESS`, `DATABASE_URL`, `RUST_LOG`)。
    /// 3. 如果任何一个环境变量未设置，它将返回一个 `AppError::Config` 错误。
    pub fn from_env() -> Result<Self, AppError> {
        // 尝试从 .env 文件加载环境变量，这对于本地开发很方便
        dotenvy::dotenv().ok();

        // 读取服务器地址
        let server_address = env::var("SERVER_ADDRESS")
            .map_err(|_| AppError::Config("必须设置 SERVER_ADDRESS".to_string()))?;
        // 读取数据库连接 URL
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| AppError::Config("必须设置 DATABASE_URL".to_string()))?;
        // 读取日志级别
        let rust_log =
            env::var("RUST_LOG").map_err(|_| AppError::Config("必须设置 RUST_LOG".to_string()))?;

        Ok(Self {
            server_address,
            database_url,
            rust_log,
        })
    }
}
