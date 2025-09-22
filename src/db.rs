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
