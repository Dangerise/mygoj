use sqlx::SqlitePool;
use std::sync::OnceLock;

pub static DB: OnceLock<SqlitePool> = OnceLock::new();

pub async fn database_connect(url: &str) -> eyre::Result<()> {
    let pool = sqlx::SqlitePool::connect(url).await?;
    DB.set(pool).unwrap();
    Ok(())
}
