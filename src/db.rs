use sqlx::{MySqlPool, Error as SqlxError};
use serde_json::Value;

pub async fn create_db_pool(database_url: &str) -> Result<MySqlPool, SqlxError> {
    MySqlPool::connect(database_url).await
}

pub async fn save_data_to_db(pool: &MySqlPool, data: &Value) -> Result<(), SqlxError> {
    // Example of saving data to the database.
    // You would replace this with your actual database logic.
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

    #[tokio::test]
    async fn test_create_db_pool_ok() {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
        let pool = create_db_pool(&database_url).await;
        assert!(pool.is_ok());
    }

    #[tokio::test]
    async fn test_create_db_pool_err() {
        let pool = create_db_pool("mysql://invalid_user:invalid_password@localhost/invalid_db").await;
        assert!(pool.is_err());
    }

    #[sqlx::test]
    async fn test_save_data_to_db(pool: MySqlPool) -> sqlx::Result<()> {
        // Create a table for the test
        sqlx::query(
            "CREATE TABLE tasks (
                id INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
                data JSON NOT NULL
            );"
        )
        .execute(&pool)
        .await?;

        let test_data = json!({ "key": "value" });
        let result = save_data_to_db(&pool, &test_data).await;
        assert!(result.is_ok());

        // Verify the data was inserted
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tasks")
            .fetch_one(&pool)
            .await?;

        assert_eq!(count, 1);

        Ok(())
    }
}
