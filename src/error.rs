use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// 应用的统一错误类型枚举。
///
/// 使用 `thiserror` 宏可以方便地为枚举的每个变体实现 `std::error::Error` trait。
/// - `#[error(...)]`: 定义了 `Display` trait 的实现，用于生成错误的文本描述。
/// - `#[from]`: 实现了 `From` trait，允许将源错误类型自动转换为 `AppError`。
#[derive(Error, Debug)]
pub enum AppError {
    /// 表示数据库操作相关的错误。
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),

    /// 表示应用配置相关的错误。
    #[error("配置错误: {0}")]
    Config(String),

    /// 表示其他所有未被明确分类的内部服务器错误。
    #[error("内部服务器错误: {0}")]
    Internal(#[from] anyhow::Error),
}

/// 为 `AppError` 实现 `IntoResponse` trait，使其可以被 axum handler 作为错误返回。
///
/// 当 handler 返回 `Result<T, AppError>` 时，如果结果是 `Err(AppError)`，
/// axum 会调用这个 `into_response` 方法将 `AppError` 转换为一个 HTTP 响应。
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // 根据错误类型匹配，决定返回的 HTTP 状态码和错误信息
        let (status, error_message) = match self {
            AppError::Database(e) => {
                // 对于数据库错误，记录详细的错误日志
                tracing::error!("数据库错误: {}", e);
                // 但为了安全，向客户端返回一个通用的错误信息
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "数据库错误".to_string(),
                )
            }
            AppError::Config(e) => {
                tracing::error!("配置错误: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "配置错误".to_string(),
                )
            }
            AppError::Internal(e) => {
                tracing::error!("内部服务器错误: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "内部服务器错误".to_string(),
                )
            }
        };

        // 将错误信息包装在 JSON 对象中作为响应体
        let body = Json(json!({ "error": error_message }));

        // 构建并返回最终的 HTTP 响应
        (status, body).into_response()
    }
}
