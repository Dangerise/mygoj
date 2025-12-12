use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Copy)]
pub struct Rid(pub u64);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum RecordStatus {
    Waiting,
    Compiling,
    CompileError(CompileError),
    Running(Vec<Option<SingleJudgeResult>>),
    Completed(AllJudgeResult),
}

impl RecordStatus {
    pub fn done(&self) -> bool {
        matches!(
            self,
            RecordStatus::CompileError(_) | RecordStatus::Completed(_)
        )
    }
}

impl std::fmt::Display for RecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Waiting => {
                write!(f, "Waiting")
            }
            Self::Compiling => {
                write!(f, "Compling")
            }
            Self::CompileError(_) => {
                write!(f, "Compile Error")
            }
            Self::Running(_) => {
                write!(f, "Running")
            }
            Self::Completed(res) => res.verdict.fmt(f),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Record {
    pub rid: Rid,
    pub pid: Pid,
    pub code: String,
    pub status: RecordStatus,
    pub timestamp: u64,
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
