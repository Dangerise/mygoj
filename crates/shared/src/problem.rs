use super::*;
use compact_str::CompactString;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct Pid(pub CompactString);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ProblemFront {
    pub pid: Pid,
    pub title: String,
    pub statement: String,
    pub time_limit: f32,
    pub memory_limit: u32,
}

impl Pid {
    pub fn new(s: &str) -> Self {
        Self(CompactString::new(s))
    }
}

impl std::fmt::Display for Pid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for Pid {
    type Err = u8;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Pid(CompactString::new(s)))
    }
}
