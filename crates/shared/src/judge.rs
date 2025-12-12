use super::*;
use compact_str::CompactString;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JudgeMessage {
    Signal(JudgeMachineSignal),
    GetProblemData(Pid),
    GetRecord(Rid),
    GetProblemFile(Pid, CompactString),
    SendCompileResult(Rid, CompileResult),
    SendSingleJudgeResult(Rid, usize, SingleJudgeResult),
    SendAllJudgeResults(Rid, AllJudgeResult),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JudgeCommand {
    Judge(Rid),
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JudgeMachineSignal {
    pub cpu_usage: u32,
    pub cpu_name: String,
    pub system_name: Option<String>,
    pub hostname: Option<String>,
    pub tasks: Vec<Rid>,
    pub uuid: uuid::Uuid,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Verdict {
    Ac,
    Wa,
    Re,
    Tle,
    Mle,
    Ce,
    Uke,
}

impl Verdict {
    pub fn priority(&self) -> u8 {
        match self {
            Verdict::Ac => 0,
            Verdict::Mle => 1,
            Verdict::Tle => 2,
            Verdict::Wa => 3,
            Verdict::Re => 4,
            Verdict::Ce => 5,
            Verdict::Uke => 6,
        }
    }
}

impl std::fmt::Display for Verdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
            Self::Ce => {
                write!(f, "Compile Error")
            }
            Self::Uke => {
                write!(f, "Unknown Error")
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CompileResult {
    Compiled,
    Error(CompileError),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SingleJudgeResult {
    pub verdict: Verdict,
    pub memory_used: u32,
    pub time_used: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AllJudgeResult {
    pub cases: Vec<SingleJudgeResult>,
    pub verdict: Verdict,
    pub memory_used: u32,
    pub max_time: u32,
    pub sum_time: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error, PartialEq)]
pub struct CompileError {
    pub message: String,
    pub exit_code: Option<i32>,
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(code) = self.exit_code {
            write!(f, "exit with {code} ")?;
        } else {
            write!(f, "no exit code ")?;
        }
        write!(f, "message = {}", &self.message)?;
        Ok(())
    }
}
