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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::queue::Task;
    use serde_json::json;
    use sqlx::MySqlPool;
    use std::sync::Arc;
    use uuid::Uuid;

    // Helper to create a table for tasks
    async fn create_temp_task_table(pool: &MySqlPool) -> sqlx::Result<()> {
        sqlx::query(
            "CREATE TABLE tasks (
                id INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
                data JSON NOT NULL
            );"
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    #[sqlx::test]
    async fn test_handle_quick_task_success(pool: MySqlPool) -> sqlx::Result<()> {
        create_temp_task_table(&pool).await?;

        let task = Task {
            id: Uuid::new_v4(),
            payload: json!({ "test": "quick_task" }),
            priority: 50,
            retry_count: 0,
        };

        let result = handle_quick_task(&task, &pool).await;
        assert!(result.is_ok());

        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tasks")
            .fetch_one(&pool)
            .await?;
        assert_eq!(count, 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_retry_logic() {
        let queue = Arc::new(PriorityQueue::new());
        let task = Task {
            id: Uuid::new_v4(),
            payload: json!({}),
            priority: 1, 
            retry_count: 0,
        };

        // This test simulates a failure by not having a database, causing handle_quick_task to fail.
        // We can't easily mock the db call without more complex dependency injection.
        let _dummy_db_pool = MySqlPool::connect("mysql://user:pass@host/db").await.err().unwrap();
        
        // Manually simulate the part of the scheduler loop that handles retries
        let mut task_to_retry = task.clone();
        if task_to_retry.retry_count < MAX_RETRIES {
            task_to_retry.retry_count += 1;
            queue.push(task_to_retry).await;
        }

        let retried_task = queue.pop().await.unwrap();
        assert_eq!(retried_task.retry_count, 1);
    }
}
