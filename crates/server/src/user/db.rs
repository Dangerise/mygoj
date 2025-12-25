use super::*;
use crate::db;
use sqlx::SqliteConnection;
use sqlx::sqlite::SqliteRow;

impl sqlx::FromRow<'_, SqliteRow> for User {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        from_json_in_row(row)
    }
}

pub async fn add_token(token: Token, uid: Uid) -> Result<(), sqlx::Error> {
    let time = chrono::Utc::now().timestamp();
    let token = token.encode();
    let uid = uid.0 as i64;
    sqlx::query!(
        "INSERT INTO tokens (token,last_time,uid) VALUES ($1,$2,$3)",
        token,
        time,
        uid
    )
    .execute(db::DB.get().unwrap())
    .await?;
    Ok(())
}

pub async fn remove_token(
    con: impl Into<Option<&mut SqliteConnection>>,
    token: Token,
) -> Result<(), sqlx::Error> {
    let token = token.encode();
    let qry = sqlx::query!("DELETE FROM tokens WHERE token=$1", token);
    if let Some(con) = con.into() {
        qry.execute(con).await?
    } else {
        qry.execute(db::DB.get().unwrap()).await?
    };
    Ok(())
}

pub async fn get_user(
    con: impl Into<Option<&mut SqliteConnection>>,
    uid: Uid,
) -> Result<Option<User>, sqlx::Error> {
    let uid = uid.0 as i64;
    let qry = sqlx::query!("SELECT (json) FROM users WHERE uid=$1", uid);
    let res = if let Some(con) = con.into() {
        qry.fetch_optional(con).await?
    } else {
        qry.fetch_optional(db::DB.get().unwrap()).await?
    };
    if let Some(res) = res {
        Ok(Some(serde_json::from_str(&res.json.unwrap()).unwrap()))
    } else {
        Ok(None)
    }
}

pub async fn find_by_email(
    con: impl Into<Option<&mut SqliteConnection>>,
    email: &str,
) -> Result<Option<Uid>, sqlx::Error> {
    let qry = sqlx::query!("SELECT (uid) FROM users WHERE email=$1", email);
    let res = if let Some(con) = con.into() {
        qry.fetch_optional(con).await?
    } else {
        qry.fetch_optional(db::DB.get().unwrap()).await?
    };
    if let Some(rec) = res {
        Ok(Some(Uid(rec.uid.unwrap() as u64)))
    } else {
        Ok(None)
    }
}

pub async fn find_by_username(
    con: impl Into<Option<&mut SqliteConnection>>,
    username: &str,
) -> Result<Option<Uid>, sqlx::Error> {
    let qry = sqlx::query!("SELECT (uid) FROM users WHERE username=$1", username);
    let res = if let Some(con) = con.into() {
        qry.fetch_optional(con).await?
    } else {
        qry.fetch_optional(db::DB.get().unwrap()).await?
    };
    if let Some(rec) = res {
        Ok(Some(Uid(rec.uid.unwrap() as u64)))
    } else {
        Ok(None)
    }
}

pub async fn find_by_token(
    con: impl Into<Option<&mut SqliteConnection>>,
    token: Token,
) -> Result<Option<Uid>, sqlx::Error> {
    let token = token.encode();
    let qry = sqlx::query!("SELECT (uid) FROM tokens WHERE token=$1", token);
    let res = if let Some(con) = con.into() {
        qry.fetch_optional(con).await?
    } else {
        qry.fetch_optional(db::DB.get().unwrap()).await?
    };
    if let Some(rec) = res {
        Ok(Some(Uid(rec.uid as u64)))
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
    let created_time = chrono::Utc::now().timestamp();
    let db = db::DB.get().unwrap();
    let mut txn = db
        .begin_with("BEGIN IMMEDIATE")
        .await
        .map_err(Either::Left)?;
    let ret = async {
        if find_by_email(&mut *txn, &email).await?.is_some(){
            return Ok(Err(ServerError::EmailExist));
        }
        if find_by_username(&mut *txn,&username).await?.is_some(){
            return Ok(Err(ServerError::UsernameExist));
        }
        let res= {
            let (email,username,password,nickname)=(email.as_str(),username.as_str(),password.as_str(),nickname.as_str());
            sqlx::query!(
                "INSERT INTO users (email,username,password,nickname,created_time) VALUES ($1,$2,$3,$4,$5)",
                email,username,password,nickname,created_time).execute(&mut *txn).await?
        };
        let uid = res.last_insert_rowid() as u64;
        let user = User {
            email,
            password,
            nickname,
            username,
            created_time,
            uid: Uid(uid),
        };
        {
            let user=serde_json::to_string(&user).unwrap();
            let uid=uid as i64;
            sqlx::query!("UPDATE users SET json=$1 WHERE uid=$2",user,uid).execute(&mut *txn).await?;
        }
        Ok(Ok(user))
    }.await;
    match ret {
        Ok(ret) => match ret {
            Ok(ret) => {
                txn.commit().await.map_err(Either::Left)?;
                Ok(ret)
            }
            Err(err) => {
                txn.rollback().await.map_err(Either::Left)?;
                Err(Either::Right(err))
            }
        },
        Err(err) => {
            txn.rollback().await.map_err(Either::Left)?;
            Err(Either::Left(err))
        }
    }
}
