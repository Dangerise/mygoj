use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JudgeSignal {
    pub cpu_usage: u32,
    pub cpu_name: String,
    pub system_name: Option<String>,
    pub hostname: Option<String>,
    pub tasks: Vec<Rid>,
    pub uuid: uuid::Uuid,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Command {
    Judge(Rid),
    Null,
}
