use super::*;
use papaya::HashMap;

#[dynamic]
static USERS: HashMap<Uid, User> = HashMap::new();

#[dynamic]
static EMAILS: HashMap<CompactString, Uid> = HashMap::new();

#[dynamic]
static USERNAMES: HashMap<CompactString, Uid> = HashMap::new();

#[dynamic]
static TOKENS: HashMap<Token, Option<Uid>> = HashMap::new();

pub async fn set_user(uid: Uid, user: User) {
    EMAILS.pin().insert(user.email.clone(), uid);
    USERNAMES.pin().insert(user.username.clone(), uid);
    USERS.pin().update(uid, |old| {
        EMAILS.pin().remove(&old.email);
        USERNAMES.pin().remove(&old.username);
        user.clone()
    });
}

pub async fn add_token(token: Token, uid: Uid) {
    TOKENS.pin().insert(token, Some(uid));
}

pub async fn remove_token(token: Token) {
    TOKENS.pin().insert(token, None);
}

pub async fn get_user(uid: Uid) -> Option<User> {
    USERS.pin().get(&uid).cloned()
}

pub async fn find_by_email(email: &str) -> Option<Uid> {
    EMAILS.pin().get(email).cloned()
}

pub async fn find_by_username(username: &str) -> Option<Uid> {
    USERNAMES.pin().get(username).cloned()
}

pub async fn find_by_token(token: Token) -> Option<Uid> {
    TOKENS.pin().get(&token).copied().flatten()
}
