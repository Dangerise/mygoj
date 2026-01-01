use super::ServerError;
use either::Either;
use futures_util::future::BoxFuture;
use sqlx::{Sqlite, SqlitePool, Transaction};
use std::sync::OnceLock;

pub static DB: OnceLock<SqlitePool> = OnceLock::new();

pub async fn database_connect(url: &str) -> eyre::Result<()> {
    let pool = sqlx::SqlitePool::connect(url).await?;
    DB.set(pool).unwrap();
    Ok(())
}

pub fn transaction<'a, F, R>(
    callback: F,
) -> BoxFuture<'a, Result<R, Either<sqlx::Error, ServerError>>>
where
    for<'c> F: FnOnce(&'c mut Transaction<'_, Sqlite>) -> BoxFuture<'c, Result<R, Either<sqlx::Error, ServerError>>>
        + 'a
        + Send
        + Sync,
    R: Send,
{
    Box::pin(async move {
        let db = DB.get().unwrap();
        let mut transaction = db
            .begin_with("BEGIN IMMEDIATE")
            .await
            .map_err(Either::Left)?;
        let ret = callback(&mut transaction).await;
        match ret {
            Ok(ret) => {
                transaction.commit().await.map_err(Either::Left)?;
                Ok(ret)
            }
            Err(err) => {
                transaction.rollback().await.map_err(Either::Left)?;
                Err(err)
            }
        }
    })
}
