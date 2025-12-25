use sqlx::{Sqlite, SqlitePool, Transaction};
use std::sync::OnceLock;

pub static DB: OnceLock<SqlitePool> = OnceLock::new();

pub async fn database_connect(url: &str) -> eyre::Result<()> {
    let pool = sqlx::SqlitePool::connect(url).await?;
    DB.set(pool).unwrap();
    Ok(())
}

pub async fn write_transaction<'a, T, F>(f: F) -> Result<T, sqlx::Error>
where
    F: AsyncFnOnce(&mut Transaction<'a, Sqlite>) -> Result<T, sqlx::Error>,
{
    let db = DB.get().unwrap();
    let mut transaction = db.begin_with("BEGIN IMMEDIATE").await?;
    let ret = f(&mut transaction).await;
    match ret {
        Ok(ret) => {
            transaction.commit().await?;
            Ok(ret)
        }
        Err(err) => {
            transaction.rollback().await?;
            Err(err)
        }
    }
}

#[tokio::test]
async fn transaction() {
    use sqlx::Row;
    use std::time::Duration;

    database_connect("sqlite::memory:").await.unwrap();

    let db = DB.get().unwrap();

    let get = async || {
        let row = sqlx::query("SELECT * FROM tab")
            .fetch_one(db)
            .await
            .unwrap();
        let g: i32 = row.get("col");
        g
    };

    sqlx::query("CREATE TABLE tab (col INT)")
        .execute(db)
        .await
        .unwrap();
    sqlx::query("INSERT INTO tab (col) VALUES ($1)")
        .bind(1)
        .execute(db)
        .await
        .unwrap();

    assert_eq!(get().await, 1);

    let clos = async |txn: &mut Transaction<'_, Sqlite>| {
        tokio::time::sleep(Duration::from_secs(3)).await;
        sqlx::query("UPDATE tab SET col=2")
            .execute(&mut **txn)
            .await?;
        Ok(())
    };

    let handle = tokio::spawn(async move {
        write_transaction(clos).await.unwrap();
    });

    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("enter");

    sqlx::query("UPDATE tab SET col=3")
        .execute(db)
        .await
        .unwrap();

    println!("done");

    assert_eq!(get().await, 3);

    handle.await.unwrap();
}
