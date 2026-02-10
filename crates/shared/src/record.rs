use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Copy, Hash, Eq)]
pub struct Rid(pub u64);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum RecordStatus {
    Waiting,
    Compiling,
    CompileError(CompileError),
    Running(Vec<Option<SingleJudgeResult>>),
    Completed(AllJudgeResult),
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum RecordFlag {
    Waiting,
    Compiling,
    Running,
    Ac,
    Wa,
    Re,
    Tle,
    Mle,
    Ce,
    Uke,
}

impl RecordFlag {
    pub fn as_str(&self) -> &'static str {
        use RecordFlag::*;
        match self {
            Waiting => "Waiting",
            Compiling => "Compiling",
            Running => "Running",
            Ac => "AC",
            Wa => "WA",
            Tle => "TLE",
            Mle => "MLE",
            Ce => "Compile Error",
            Re => "RE",
            Uke => "Unknown Error",
        }
    }
}

impl RecordStatus {
    pub fn done(&self) -> bool {
        matches!(
            self,
            RecordStatus::CompileError(_) | RecordStatus::Completed(_)
        )
    }
    pub fn flag(&self) -> RecordFlag {
        match self {
            RecordStatus::Waiting => RecordFlag::Waiting,
            RecordStatus::Running(_) => RecordFlag::Running,
            RecordStatus::Completed(x) => x.verdict.flag(),
            RecordStatus::CompileError(_) => RecordFlag::Ce,
            RecordStatus::Compiling => RecordFlag::Compiling,
        }
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
    pub uid: Uid,
    pub pid: Pid,
    pub code: String,
    pub status: RecordStatus,
    pub time: i64,
}

impl Record {
    pub fn flag(&self) -> RecordFlag {
        self.status.flag()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum RecordMessage {
    Compiling,
    Compiled(usize),
    CompileError(CompileError),
    NewSingleResult(usize, SingleJudgeResult),
    Completed(AllJudgeResult),
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

#[cfg(feature = "server")]
mod native {
    use super::Record;
    use crate::from_json_in_row;
    use sqlx::sqlite::SqliteRow;
    use sqlx::{FromRow, SqlitePool};
    impl FromRow<'_, SqliteRow> for Record {
        fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
            from_json_in_row(row)
        }
    }
    impl Record {
        pub async fn insert_db(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
            sqlx::query(
                "INSERT INTO records (rid,uid,pid,flag,time,json) VALUES ($1,$2,$3,$4,$5,$6)",
            )
            .bind(self.rid.0 as i64)
            .bind(self.uid.0 as i64)
            .bind(self.pid.0.as_str())
            .bind(self.flag().as_str())
            .bind(self.time)
            .bind(serde_json::to_string(self).unwrap())
            .execute(pool)
            .await?;
            Ok(())
        }
    }
}
