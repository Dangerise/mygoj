use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq)]
pub struct Submission {
    pub code: String,
    pub pid: Pid,
}
