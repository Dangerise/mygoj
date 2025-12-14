use super::*;
use compact_str::CompactString;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq, Copy)]
pub struct Uid(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
pub struct UserRegistration {
    pub email: CompactString,
    pub password: CompactString,
    pub nickname: CompactString,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
pub struct LoginedUser {
    pub uid: Uid,
    pub email: CompactString,
    pub nickname: CompactString,
    pub token: Token,
}
