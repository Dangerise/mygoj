use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Copy)]
pub struct Rid(pub u64);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum JudgeStatus {
    Waiting,
    Judging,
    Ac,
    Wa,
    Tle,
    Mle,
    Re,
    Uke,
}

impl std::fmt::Display for JudgeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Waiting => {
                write!(f, "Waiting")
            }
            Self::Judging => {
                write!(f, "Judging")
            }
            Self::Ac => {
                write!(f, "Accepted")
            }
            Self::Wa => {
                write!(f, "Wrong Answer")
            }
            Self::Re => {
                write!(f, "Runtime Error")
            }
            Self::Tle => {
                write!(f, "Time Limit Exceed")
            }
            Self::Mle => {
                write!(f, "Memory Limit Exceed")
            }
            Self::Uke => {
                write!(f, "Unknown Error")
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Record {
    pub rid: Rid,
    pub pid: Pid,
    pub code: String,
    pub status: JudgeStatus,
}

impl std::fmt::Display for Rid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

use std::str::FromStr;
impl FromStr for Rid {
    type Err = <u64 as FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Rid(u64::from_str(s)?))
    }
}
