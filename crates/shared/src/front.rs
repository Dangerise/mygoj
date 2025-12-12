use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub enum FrontMessage {
    GetProblemFront(Pid),
    GetRecord(Rid),
    Submit(Submission),
    CheckJudgeMachines,
}
