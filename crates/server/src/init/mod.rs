use futures_util::StreamExt;
use sqlx::{Executor, SqlitePool};
use std::io::{Error as IoError, ErrorKind};
use std::path::Path;
use tokio::fs;

use crate::user::User;
use shared::user::Uid;

#[cfg(debug_assertions)]
mod embed;

pub async fn init_fs(path: impl AsRef<Path>) -> eyre::Result<()> {
    let path = path.as_ref();
    assert!(path.is_dir());

    #[cfg(debug_assertions)]
    {
        if fs::try_exists(path).await? {
            fs::remove_dir_all(path).await?;
        }
    }

    if fs::try_exists(path).await? {
        return Err(IoError::new(ErrorKind::AlreadyExists, "directory already exists").into());
    }
    fs::create_dir_all(path).await?;

    #[cfg(debug_assertions)]
    {
        embed::with_fs(path).await?;
    }

    Ok(())
}

pub async fn init_db(path: impl AsRef<Path>) -> eyre::Result<()> {
    let path = path.as_ref().as_os_str().to_str().unwrap();
    tracing::info!("create database at {path}");

    #[cfg(debug_assertions)]
    {
        if fs::try_exists(path).await? {
            fs::remove_file(path).await?;
        }
    }
    if fs::try_exists(path).await? {
        return Err(IoError::new(ErrorKind::AlreadyExists, "database already exists").into());
    }
    fs::write(path, "").await?;

    let pool = SqlitePool::connect(path).await?;
    let mut stream = pool.execute_many(include_str!("../sql/create.sql"));
    while let Some(ret) = stream.next().await {
        let _ = ret?;
    }

    #[cfg(debug_assertions)]
    {
        embed::with_db(&pool).await?;
    }

    Ok(())
}
