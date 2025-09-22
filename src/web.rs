use crate::error::AppError;
use crate::queue::{PriorityQueue, Task};
use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::post,
    Json, Router,
};
use serde::Deserialize;
use sqlx::MySqlPool;
use std::sync::Arc;
use tower_http::request_id::{MakeRequestUuid, SetRequestIdLayer};
use uuid::Uuid;

/// 应用状态，包含数据库连接池和任务队列。
/// `#[derive(Clone)]` 允许在多个 handler 之间安全地共享 `AppState`。
#[derive(Clone)]
pub struct AppState {
    pub db_pool: MySqlPool,
    pub queue: Arc<PriorityQueue>,
}

/// 创建任务的请求体 (payload)。
#[derive(Deserialize)]
pub struct CreateTaskPayload {
    payload: serde_json::Value,
    priority: u8,
}

/// `POST /tasks` 的 handler。
///
/// 从请求体中接收任务数据，创建一个 `Task` 并将其推入优先级队列。
/// - `State(state)`: 提取共享的应用状态 `AppState`。
/// - `Json(payload)`: 将请求体 JSON 反序列化为 `CreateTaskPayload`。
async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskPayload>,
) -> Result<StatusCode, AppError> {
    let task = Task {
        id: Uuid::new_v4(),
        payload: payload.payload,
        priority: payload.priority,
        retry_count: 0,
    };

    // 将任务推入队列
    state.queue.push(task).await;

    // 返回 202 Accepted 状态码，表示请求已被接受处理
    Ok(StatusCode::ACCEPTED)
}

/// 创建并配置 API 路由。
pub fn api_router(app_state: AppState) -> Router {
    Router::new()
        // 定义 `/tasks` 路由，仅接受 POST 请求，并由 `create_task` handler 处理
        .route("/tasks", post(create_task))
        // 将应用状态 `app_state` 注入到所有路由的 handler 中
        .with_state(app_state)
        // 添加中间件层，用于生成和设置请求ID
        .layer(SetRequestIdLayer::new(
            header::HeaderName::from_static("x-request-id"),
            MakeRequestUuid,
        ))
        // 添加自定义中间件，用于将请求ID集成到日志中
        .layer(middleware::from_fn(request_id_middleware))
}

/// 自定义中间件，用于从请求头中提取请求ID并将其添加到日志的 span 中。
async fn request_id_middleware(request: Request, next: Next) -> Response {
    // 从请求头 "x-request-id" 中获取请求ID，如果不存在则生成一个
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    // 创建一个新的日志 span，并附带请求ID
    let span = tracing::info_span!("http_request", request_id = %request_id);
    // 进入 span，后续的日志都将包含此 span 的信息
    let _enter = span.enter();
    // 调用下一个中间件或 handler
    next.run(request).await
}
