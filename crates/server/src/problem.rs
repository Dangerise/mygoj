use serde::{Deserialize, Serialize};

pub use shared::problem::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Testcase {
    pub input: String,
    pub output: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Problem {
    pub front: ProblemFront,
    pub testcases: Vec<Testcase>,
}
