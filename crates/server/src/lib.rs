mod front;
mod judge;
mod problem;
mod record;
mod user;

pub mod init;
pub mod serve;

use shared::error::ServerError;
use std::path::PathBuf;
use std::sync::OnceLock;

static DB: OnceLock<sqlx::SqlitePool> = OnceLock::new();

pub async fn database_connect(url: &str) -> eyre::Result<()> {
    let pool = sqlx::SqlitePool::connect(url).await?;
    DB.set(pool).unwrap();
    Ok(())
}

#[track_caller]
pub fn storage_dir() -> PathBuf {
    dirs::home_dir().unwrap().join("mygoj")
}
