use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct Pid(pub String);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ProblemFront {
    #[serde(default)]
    pub pid: Pid,
    pub title: String,
    pub statement: String,
}

impl std::fmt::Display for Pid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for Pid {
    type Err = u8;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Pid(s.to_string()))
    }
}
