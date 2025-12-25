mod cache;
mod db;

use super::ServerError;
use compact_str::CompactString;
use either::Either;
use serde::{Deserialize, Serialize};
use shared::from_json_in_row;
use shared::token::Token;
use shared::user::*;
use static_init::dynamic;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub uid: Uid,
    pub email: CompactString,
    pub username: CompactString,
    pub password: CompactString,
    pub nickname: CompactString,
    pub created_time: i64,
}

async fn get_user(uid: Uid) -> Result<Option<User>, ServerError> {
    if let Some(user) = cache::get_user(uid).await {
        return Ok(Some(user));
    }
    db::get_user(None, uid)
        .await
        .map_err(ServerError::into_internal)
}

pub async fn user_register(reg: UserRegistration) -> Result<Uid, ServerError> {
    let UserRegistration {
        email,
        password,
        username,
        ..
    } = &reg;
    if email.len() > 50 || password.len() > 50 || username.contains("@") {
        return Err(ServerError::Fuck);
    }

    let user = db::register(reg)
        .await
        .map_err(|err| err.map_left(ServerError::into_internal).either_into())?;

    // let user: User = todo!();
    let uid = user.uid;

    cache::update_user(uid, user).await;

    Ok(uid)
}

async fn find_by_email(email: &CompactString) -> Result<Option<Uid>, ServerError> {
    if let Some(uid) = cache::find_by_email(email).await {
        return Ok(Some(uid));
    }
    db::find_by_email(None, &email)
        .await
        .map_err(ServerError::into_internal)
}

async fn find_by_username(email: &CompactString) -> Result<Option<Uid>, ServerError> {
    if let Some(uid) = cache::find_by_username(email).await {
        return Ok(Some(uid));
    }
    db::find_by_username(None, &email)
        .await
        .map_err(ServerError::into_internal)
}

async fn find_by_token(token: Token) -> Result<Option<Uid>, ServerError> {
    if let Some(uid) = cache::find_by_token(token).await {
        return Ok(Some(uid));
    }
    db::find_by_token(None, token)
        .await
        .map_err(ServerError::into_internal)
}

pub async fn user_login(
    ident: CompactString,
    password: CompactString,
) -> Result<Token, ServerError> {
    let uid = if ident.contains("@") {
        find_by_email(&ident).await?
    } else {
        find_by_username(&ident).await?
    }
    .ok_or(ServerError::UserNotFound)?;
    let user = get_user(uid).await?.unwrap();
    if user.password != password {
        return Err(ServerError::PasswordWrong);
    }

    let token = Token::new();
    cache::add_token(token, uid).await;
    db::add_token(token, uid)
        .await
        .map_err(ServerError::into_internal)?;

    Ok(token)
}

pub async fn get_user_login(token: Token) -> Result<LoginedUser, ServerError> {
    let uid = find_by_token(token).await?.ok_or(ServerError::LoginOutDated)?;
    let user = get_user(uid).await?.unwrap();
    let logined = LoginedUser {
        uid: user.uid,
        nickname: user.nickname,
        email: user.email,
    };
    Ok(logined)
}

pub async fn remove_token(token: Token) -> Result<(), ServerError> {
    cache::remove_token(token).await;
    db::remove_token(None, token).await.unwrap();
    Ok(())
}
