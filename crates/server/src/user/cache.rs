use super::*;
use moka::future::Cache;

const LIMIT: u64 = 1 << 15;

#[dynamic]
static USERS: Cache<Uid, User> = Cache::new(LIMIT);

#[dynamic]
static EMAILS: Cache<CompactString, Uid> = Cache::new(LIMIT);

#[dynamic]
static USERNAMES: Cache<CompactString, Uid> = Cache::new(LIMIT);

#[dynamic]
static TOKENS: Cache<Token, Option<Uid>> = Cache::new(LIMIT);

pub async fn update_user(uid: Uid, user: User) {
    EMAILS.insert(user.email.clone(), uid).await;
    USERNAMES.insert(user.username.clone(), uid).await;
    USERS.insert(uid, user).await;
}

pub async fn add_token(token: Token, uid: Uid) {
    TOKENS.insert(token, Some(uid)).await
}

pub async fn remove_token(token: Token) {
    TOKENS.insert(token, None).await
}

pub async fn get_user(uid: Uid) -> Option<User> {
    USERS.get(&uid).await
}

pub async fn find_by_email(email: &CompactString) -> Option<Uid> {
    EMAILS.get(email).await
}

pub async fn find_by_username(username: &CompactString) -> Option<Uid> {
    USERNAMES.get(username).await
}

pub async fn find_by_token(token: Token) -> Option<Uid> {
    TOKENS.get(&token).await.flatten()
}
