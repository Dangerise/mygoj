use super::*;
use compact_str::CompactString;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub enum FrontMessage {
    GetProblemEditable(Pid),
    GetProblemFront(Pid),
    GetProblemFiles(Pid),
    GetProblemFileMeta(Pid, CompactString),
    RequireProblemFileDownloadToken(Pid, CompactString),
    GetRecord(Rid),
    Submit(Submission),
    GetLoginedUser,
    RegisterUser(UserRegistration),
    CheckJudgeMachines,
}
