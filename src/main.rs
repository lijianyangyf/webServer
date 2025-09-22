mod config;
mod db;
mod error;
mod logging;
mod queue;
mod scheduler;
mod web;

use crate::config::Config;
use crate::db::create_db_pool;
use crate::error::AppError;
use crate::queue::PriorityQueue;
use crate::scheduler::run_scheduler;
use crate::web::{api_router, AppState};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let config = Config::from_env()?;
    let _guard = logging::init_logging(&config, "logs")?;

    let db_pool = create_db_pool(&config.database_url).await?;
    let queue = Arc::new(PriorityQueue::new());

    let app_state = AppState {
        db_pool: db_pool.clone(),
        queue: queue.clone(),
    };

    tokio::spawn(run_scheduler(queue, db_pool));

    let app = api_router(app_state);

    let listener = TcpListener::bind(&config.server_address).await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("signal received, starting graceful shutdown");
}