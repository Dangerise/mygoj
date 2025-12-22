use futures_util::StreamExt;
use sqlx::{Executor, SqlitePool};
use std::io::{Error as IoError, ErrorKind};
use std::path::Path;
use tokio::fs;

use crate::user::User;
use shared::user::Uid;

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
    let mut stream = pool.execute_many(include_str!("sql/create.sql"));
    while let Some(ret) = stream.next().await {
        let _ = ret?;
    }

    #[cfg(debug_assertions)]
    {
        let author = User {
            email: "dangerise@qq.com".into(),
            password: "1234".into(),
            created_time: 0,
            nickname: "Dangerise".into(),
            username: "Dangerise".into(),
            uid: Uid(1),
        };
        author.insert_db(&pool).await?;
    }

    Ok(())
}
