use super::*;
use crate::db;
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, Sqlite, SqliteConnection, Transaction};

impl sqlx::FromRow<'_, SqliteRow> for User {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        from_json_in_row(row)
    }
}

pub async fn add_token(token: Token, uid: Uid) -> Result<(), sqlx::Error> {
    let time = chrono::Utc::now().timestamp();
    sqlx::query("INSERT INTO tokens (token,created_time,uid) VALUES ($1,$2,$3)")
        .bind(token.encode())
        .bind(time)
        .bind(uid.0 as i64)
        .execute(db::DB.get().unwrap())
        .await?;
    Ok(())
}

pub async fn get_user(
    con: impl Into<Option<&mut SqliteConnection>>,
    uid: Uid,
) -> Result<Option<User>, sqlx::Error> {
    let qry = sqlx::query_as("SELECT (json) FROM users WHERE uid=$1").bind(uid.0 as i64);
    let res = if let Some(con) = con.into() {
        qry.fetch_optional(con).await?
    } else {
        qry.fetch_optional(db::DB.get().unwrap()).await?
    };
    if let Some(user) = res {
        Ok(Some(user))
    } else {
        Ok(None)
    }
}

pub async fn find_by_email(
    con: impl Into<Option<&mut SqliteConnection>>,
    email: &str,
) -> Result<Option<Uid>, sqlx::Error> {
    let qry = sqlx::query("SELECT (uid) FROM users WHERE email=$1").bind(email);
    let res = if let Some(con) = con.into() {
        qry.fetch_optional(con).await?
    } else {
        qry.fetch_optional(db::DB.get().unwrap()).await?
    };
    if let Some(row) = res {
        let uid: i64 = row.get(0);
        Ok(Some(Uid(uid as u64)))
    } else {
        Ok(None)
    }
}

pub async fn find_by_username(
    con: impl Into<Option<&mut SqliteConnection>>,
    username: &str,
) -> Result<Option<Uid>, sqlx::Error> {
    let qry = sqlx::query("SELECT (uid) FROM users WHERE username=$1").bind(username);
    let res = if let Some(con) = con.into() {
        qry.fetch_optional(con).await?
    } else {
        qry.fetch_optional(db::DB.get().unwrap()).await?
    };
    if let Some(row) = res {
        let uid: i64 = row.get(0);
        Ok(Some(Uid(uid as u64)))
    } else {
        Ok(None)
    }
}

pub async fn find_by_token(
    con: impl Into<Option<&mut SqliteConnection>>,
    token: Token,
) -> Result<Option<Uid>, sqlx::Error> {
    let qry = sqlx::query("SELECT (uid) FROM tokens WHERE token=$1").bind(token.encode());
    let res = if let Some(con) = con.into() {
        qry.fetch_optional(con).await?
    } else {
        qry.fetch_optional(db::DB.get().unwrap()).await?
    };
    if let Some(row) = res {
        let uid: i64 = row.get(0);
        Ok(Some(Uid(uid as u64)))
    } else {
        Ok(None)
    }
}

pub async fn register(
    UserRegistration {
        email,
        password,
        nickname,
        username,
    }: UserRegistration,
) -> Result<User, Either<sqlx::Error, ServerError>> {
    tokio::task::spawn_local(async{
            let created_time = chrono::Utc::now().timestamp();
            let ret: Result<User, ServerError> = db::write_transaction(async |txn| {
                if find_by_email(&mut **txn, &email).await?.is_some(){
                    return Ok(Err(ServerError::EmailExist));
                }
                if find_by_username(&mut **txn,&username).await?.is_some(){
                    return Ok(Err(ServerError::UsernameExist));
                }
                let res = sqlx::query(
                    "INSERT INTO users (email,username,password,nickname,created_time) VALUES ($1,$2,$3,$4,$5)",
                )
                .bind(email.as_str())
                .bind(username.as_str())
                .bind(password.as_str())
                .bind(nickname.as_str())
                .bind(created_time)
                .execute(&mut **txn)
                .await?;
                let uid = res.last_insert_rowid() as u64;
                let user = User {
                    email,
                    password,
                    nickname,
                    username,
                    created_time,
                    uid: Uid(uid),
                };
                sqlx::query("UPDATE users (json=$1) WHERE uid=$2")
                    .bind(serde_json::to_string(&user).unwrap())
                    .bind(uid as i64)
                    .execute(&mut **txn)
                    .await?;
                Ok(Ok(user))
            })
            .await
            .map_err(Either::Left)?;
            ret.map_err(Either::Right)
    }).await.unwrap()
}
