use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum JudgeStatus {
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
    pub rid: u64,
    pub pid: String,
    pub code: String,
    pub status: JudgeStatus,
}
