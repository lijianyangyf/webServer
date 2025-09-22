use serde_json::Value;
use sqlx::{Error as SqlxError, MySqlPool};

/// 根据提供的数据库 URL 创建一个 `MySqlPool` 连接池。
pub async fn create_db_pool(database_url: &str) -> Result<MySqlPool, SqlxError> {
    MySqlPool::connect(database_url).await
}

/// 将数据保存到数据库。
/// 这是一个示例函数，实际应用中应替换为具体的业务逻辑。
pub async fn save_data_to_db(pool: &MySqlPool, data: &Value) -> Result<(), SqlxError> {
    // 示例：将 JSON 数据插入到 `tasks` 表的 `data` 字段。
    // 在实际应用中，您需要根据自己的表结构和需求来修改此查询。
    sqlx::query("INSERT INTO tasks (data) VALUES (?)")
        .bind(data)
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenvy::dotenv;
    use serde_json::json;
    use std::env;

    /// 测试 `create_db_pool` 函数能否成功创建一个数据库连接池。
    /// 需要在 `.env` 文件中配置 `DATABASE_URL`。
    #[tokio::test]
    #[ignore]
    async fn test_create_db_pool_ok() {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
        let pool = create_db_pool(&database_url).await;
        assert!(pool.is_ok());
    }

    /// 测试 `create_db_pool` 在提供无效连接字符串时是否会返回错误。
    #[tokio::test]
    async fn test_create_db_pool_err() {
        let pool =
            create_db_pool("mysql://invalid_user:invalid_password@localhost/invalid_db").await;
        assert!(pool.is_err());
    }

    /// 使用 `sqlx::test` 宏进行集成测试，该宏会自动处理数据库的建立和清理。
    /// 测试 `save_data_to_db` 函数是否能成功将数据写入数据库。
    #[sqlx::test]
    #[ignore]
    async fn test_save_data_to_db(pool: MySqlPool) -> sqlx::Result<()> {
        // 为测试创建一个临时表 `tasks`
        sqlx::query(
            "CREATE TABLE tasks (
                id INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
                data JSON NOT NULL
            );",
        )
        .execute(&pool)
        .await?;

        // 准备测试数据并调用函数
        let test_data = json!({ "key": "value" });
        let result = save_data_to_db(&pool, &test_data).await;
        assert!(result.is_ok());

        // 验证数据是否已成功插入
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tasks")
            .fetch_one(&pool)
            .await?;

        assert_eq!(count, 1);

        Ok(())
    }
}
