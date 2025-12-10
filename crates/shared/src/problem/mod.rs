use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ProblemFront {
    #[serde(default)]
    pub pid: String,
    pub title: String,
    pub statement: String,
}
