use super::*;
use compact_str::CompactString;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq, Copy)]
pub struct Uid(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
pub struct UserRegistration {
    pub email: CompactString,
    pub password: CompactString,
    pub nickname: CompactString,
    pub username: CompactString,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
pub struct LoginedUser {
    pub uid: Uid,
    pub email: CompactString,
    pub nickname: CompactString,
    pub privilege: Privilege,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
pub struct Privilege {
    pub edit_problems: bool,
    pub enter_site: bool,
}

impl Privilege {
    pub const DEFAULT: Self = Self {
        edit_problems: false,
        enter_site: true,
    };
    pub const ALL: Self = Self {
        edit_problems: true,
        enter_site: true,
    };
}

impl Default for Privilege {
    fn default() -> Self {
        Self::DEFAULT
    }
}
