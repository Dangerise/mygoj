use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub enum FrontMessage {
    GetProblemEditable(Pid),
    GetProblemFront(Pid),
    GetProblemFiles(Pid),
    GetRecord(Rid),
    Submit(Submission),
    GetLoginedUser,
    RegisterUser(UserRegistration),
    CheckJudgeMachines,
}
