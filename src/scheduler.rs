use crate::db::save_data_to_db;
use crate::queue::{PriorityQueue, Task};
use sqlx::MySqlPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

const MAX_RETRIES: u8 = 3;

async fn handle_quick_task(task: &Task, db_pool: &MySqlPool) -> Result<(), anyhow::Error> {
    tracing::info!(task_id = %task.id, "Handling quick task");
    save_data_to_db(db_pool, &task.payload).await?;
    Ok(())
}

async fn handle_slow_task(task: Task, db_pool: MySqlPool) {
    tracing::info!(task_id = %task.id, "Handling slow task");
    // Simulate a long-running task
    sleep(Duration::from_secs(5)).await;
    if let Err(e) = save_data_to_db(&db_pool, &task.payload).await {
        tracing::error!(task_id = %task.id, "Failed to handle slow task: {}", e);
    }
}

pub async fn run_scheduler(queue: Arc<PriorityQueue>, db_pool: MySqlPool) {
    tracing::info!("Scheduler started");
    loop {
        if let Some(mut task) = queue.pop().await {
            tracing::debug!(task_id = %task.id, "Popped task from queue");
            let db_pool_clone = db_pool.clone();
            let queue_clone = queue.clone();

            // Simple logic to differentiate tasks
            if task.priority > 100 {
                tokio::spawn(async move {
                    handle_slow_task(task, db_pool_clone).await;
                });
            } else {
                match handle_quick_task(&task, &db_pool_clone).await {
                    Ok(_) => tracing::info!(task_id = %task.id, "Quick task handled successfully"),
                    Err(e) => {
                        tracing::error!(task_id = %task.id, "Failed to handle quick task: {}. Retrying...", e);
                        if task.retry_count < MAX_RETRIES {
                            task.retry_count += 1;
                            queue_clone.push(task).await;
                        } else {
                            tracing::error!(task_id = %task.id, "Task failed after {} retries", MAX_RETRIES);
                        }
                    }
                }
            }
        } else {
            sleep(Duration::from_secs(1)).await;
        }
    }
}
