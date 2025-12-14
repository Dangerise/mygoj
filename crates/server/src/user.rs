use super::Error;
use compact_str::CompactString;
use dashmap::{DashMap, DashSet};
use shared::token::Token;
use shared::user::*;
use static_init::dynamic;

#[derive(Debug)]
pub struct User {
    pub uid: Uid,
    pub email: CompactString,
    pub password: CompactString,
    pub nickname: CompactString,
}

#[dynamic]
static mut LAST_UID: Uid = Uid(0);

#[dynamic]
static USERS: DashMap<Uid, User> = DashMap::new();

#[dynamic]
static EMAILS: DashMap<CompactString, Uid> = DashMap::new();

#[dynamic]
static EMAILS_SET: DashSet<CompactString> = DashSet::new();

pub async fn register_user(
    UserRegistration {
        email,
        password,
        nickname,
    }: UserRegistration,
) -> eyre::Result<Uid> {
    eyre::ensure!(email.len() <= 50, "email too long");
    eyre::ensure!(password.len() <= 50, "pwd too long");

    if !EMAILS_SET.insert(email.clone()) {
        return Err(eyre::eyre!("repeated email"));
    }

    let mut last_uid = LAST_UID.write();
    last_uid.0 += 1;
    let uid = *last_uid;

    drop(last_uid);

    let ret = EMAILS.insert(email.clone(), uid);

    assert!(ret.is_none());

    let ret = USERS.insert(
        uid,
        User {
            uid,
            email,
            password,
            nickname,
        },
    );

    assert!(ret.is_none());

    Ok(uid)
}

#[dynamic]
static LOGIN_STATES: DashMap<Token, Uid> = DashMap::new();

pub async fn user_login(
    email: CompactString,
    password: CompactString,
) -> Result<LoginedUser, Error> {
    let uid = EMAILS.get(&email).ok_or(Error::UserNotFound)?;
    let user = USERS.get(&uid).unwrap();
    if user.password == password {
        let uid = *uid;
        let token = Token::new();
        let ret = LOGIN_STATES.insert(token, uid);
        assert!(ret.is_none());
        Ok(LoginedUser {
            uid,
            email: user.email.clone(),
            nickname: user.nickname.clone(),
            token,
        })
    } else {
        Err(Error::PasswordWrong)
    }
}

pub async fn get_user_login(token: Token) -> Option<Uid> {
    LOGIN_STATES.get(&token).map(|x| *x)
}
