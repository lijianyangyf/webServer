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

#[derive(Clone)]
pub struct AppState {
    pub db_pool: MySqlPool,
    pub queue: Arc<PriorityQueue>,
}

#[derive(Deserialize)]
pub struct CreateTaskPayload {
    payload: serde_json::Value,
    priority: u8,
}

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

    state.queue.push(task).await;

    Ok(StatusCode::ACCEPTED)
}

pub fn api_router(app_state: AppState) -> Router {
    Router::new()
        .route("/tasks", post(create_task))
        .with_state(app_state)
        .layer(SetRequestIdLayer::new(
            header::HeaderName::from_static("x-request-id"),
            MakeRequestUuid,
        ))
        .layer(middleware::from_fn(request_id_middleware))
}

async fn request_id_middleware(request: Request, next: Next) -> Response {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    let span = tracing::info_span!("http_request", request_id = %request_id);
    let _enter = span.enter();
    next.run(request).await
}
