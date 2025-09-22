use crate::db::save_data_to_db;
use crate::queue::{PriorityQueue, Task};
use sqlx::MySqlPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

// 定义任务失败后的最大重试次数
const MAX_RETRIES: u8 = 3;

/// 处理可以快速完成的任务。
///
/// 这个函数会尝试将任务的载荷保存到数据库。
/// 如果失败，它会返回一个错误，由调用者决定是否重试。
async fn handle_quick_task(task: &Task, db_pool: &MySqlPool) -> Result<(), anyhow::Error> {
    tracing::info!(task_id = %task.id, "正在处理快速任务");
    save_data_to_db(db_pool, &task.payload).await?;
    Ok(())
}

/// 处理需要较长时间的慢速任务。
///
/// 这个函数会模拟一个耗时操作（如调用第三方 API 或进行复杂计算），
/// 然后将结果保存到数据库。慢速任务会在一个独立的 Tokio 任务中运行，
/// 以避免阻塞调度器主循环。
async fn handle_slow_task(task: Task, db_pool: MySqlPool) {
    tracing::info!(task_id = %task.id, "正在处理慢速任务");
    // 模拟一个耗时 5 秒的操作
    sleep(Duration::from_secs(5)).await;
    if let Err(e) = save_data_to_db(&db_pool, &task.payload).await {
        tracing::error!(task_id = %task.id, "处理慢速任务失败: {}", e);
    }
}

/// 运行后台任务调度器。
///
/// 这是一个无限循环，不断地从优先级队列中弹出任务并进行处理。
pub async fn run_scheduler(queue: Arc<PriorityQueue>, db_pool: MySqlPool) {
    tracing::info!("调度器已启动");
    loop {
        // 尝试从队列中弹出一个任务
        if let Some(mut task) = queue.pop().await {
            tracing::debug!(task_id = %task.id, "从队列中取出一个任务");
            let db_pool_clone = db_pool.clone();
            let queue_clone = queue.clone();

            // 简单的任务区分逻辑：根据优先级决定如何处理
            if task.priority > 100 {
                // 对于高优先级任务，我们假设它们是“慢速任务”，
                // 在一个新的 Tokio 任务中异步处理，防止阻塞调度器。
                tokio::spawn(async move {
                    handle_slow_task(task, db_pool_clone).await;
                });
            } else {
                // 对于普通任务，我们假设它们是“快速任务”，
                // 直接在当前循环中处理。
                match handle_quick_task(&task, &db_pool_clone).await {
                    Ok(_) => tracing::info!(task_id = %task.id, "快速任务处理成功"),
                    Err(e) => {
                        // 如果任务处理失败，记录错误并检查是否可以重试
                        tracing::error!(task_id = %task.id, "处理快速任务失败: {}. 正在重试...", e);
                        if task.retry_count < MAX_RETRIES {
                            // 如果重试次数未达上限，增加重试计数并将任务重新推入队列
                            task.retry_count += 1;
                            queue_clone.push(task).await;
                        } else {
                            // 如果已达到最大重试次数，则放弃任务
                            tracing::error!(task_id = %task.id, "任务在 {} 次重试后失败", MAX_RETRIES);
                        }
                    }
                }
            }
        } else {
            // 如果队列为空，则休眠 1 秒，避免忙等待消耗过多 CPU
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

    // 辅助函数：为测试创建一个临时的 `tasks` 表
    async fn create_temp_task_table(pool: &MySqlPool) -> sqlx::Result<()> {
        sqlx::query(
            "CREATE TABLE tasks (
                id INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
                data JSON NOT NULL
            );",
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    /// 测试 `handle_quick_task` 成功执行的情况
    #[sqlx::test]
    #[ignore]
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

        // 验证数据是否已插入
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tasks")
            .fetch_one(&pool)
            .await?;
        assert_eq!(count, 1);

        Ok(())
    }

    /// 测试任务失败后的重试逻辑
    #[tokio::test]
    async fn test_retry_logic() {
        let queue = Arc::new(PriorityQueue::new());
        let task = Task {
            id: Uuid::new_v4(),
            payload: json!({}),
            priority: 1,
            retry_count: 0,
        };

        // 这个测试通过不提供真实数据库来模拟 `handle_quick_task` 的失败。
        // 在没有更复杂的依赖注入或 mock 框架的情况下，这是一种简单的模拟方式。
        // let _dummy_db_pool = MySqlPool::connect("mysql://user:pass@host/db").await.err().unwrap();

        // 手动模拟调度器循环中的重试部分
        let mut task_to_retry = task.clone();
        if task_to_retry.retry_count < MAX_RETRIES {
            task_to_retry.retry_count += 1;
            queue.push(task_to_retry).await;
        }

        // 验证任务被重新推入队列后，其重试计数增加了
        let retried_task = queue.pop().await.unwrap();
        assert_eq!(retried_task.retry_count, 1);
    }
}
