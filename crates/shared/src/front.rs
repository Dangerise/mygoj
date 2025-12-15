use super::*;
use compact_str::CompactString;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub enum FrontMessage {
    GetProblemFront(Pid),
    GetRecord(Rid),
    Submit(Submission),
    LoginUser(CompactString, CompactString),
    GetLoginedUser,
    RegisterUser(UserRegistration),
    CheckJudgeMachines,
    Logout,
}
