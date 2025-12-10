use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Submission {
    pub code: String,
    pub pid: Pid,
}
