// 模块声明
mod config;
mod db;
mod error;
mod logging;
mod queue;
mod scheduler;
mod web;

// 引入外部依赖和内部模块
use crate::config::Config;
use crate::db::create_db_pool;
use crate::error::AppError;
use crate::queue::PriorityQueue;
use crate::scheduler::run_scheduler;
use crate::web::{api_router, AppState};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;

/// 应用主入口
#[tokio::main]
async fn main() -> Result<(), AppError> {
    // 从环境变量加载配置
    let config = Config::from_env()?;
    // 初始化日志系统
    let _guard = logging::init_logging(&config, "logs")?;

    // 创建数据库连接池
    let db_pool = create_db_pool(&config.database_url).await?;
    // 创建一个带引用计数的、线程安全的优先级队列
    let queue = Arc::new(PriorityQueue::new());

    // 创建应用状态，用于在 axum handler 中共享
    let app_state = AppState {
        db_pool: db_pool.clone(),
        queue: queue.clone(),
    };

    // 在后台 Tokio 任务中运行调度器
    tokio::spawn(run_scheduler(queue, db_pool));

    // 创建 axum 路由
    let app = api_router(app_state);

    // 绑定服务器地址并启动
    let listener = TcpListener::bind(&config.server_address).await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal()) // 设置优雅停机
        .await
        .unwrap();

    Ok(())
}

/// 监听停机信号，用于实现优雅停机
async fn shutdown_signal() {
    // 监听 Ctrl+C 信号
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    // 在 Unix 系统上监听终止信号
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    // 在非 Unix 系统上，terminate future 永远不会完成
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    // 等待任一信号
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("signal received, starting graceful shutdown");
}