use super::*;
use compact_str::CompactString;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default, Eq, Hash)]
pub struct Pid(pub CompactString);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ProblemFront {
    pub pid: Pid,
    pub owner: Option<Uid>,
    pub title: CompactString,
    pub statement: String,
    pub time_limit: u32,
    pub memory_limit: u32,
    pub public_files: Vec<CompactString>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Testcase {
    pub input_file: CompactString,
    pub output_file: CompactString,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ProblemFile {
    pub path: CompactString,
    pub uuid: Uuid,
    pub is_public: bool,
    pub size: u64,
    pub last_modified: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ProblemData {
    pub pid: Pid,
    pub testcases: Vec<Testcase>,
    #[serde(default)]
    pub files: Vec<ProblemFile>,
    pub time_limit: u32,
    pub memory_limit: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ProblemEditable {
    pub owner: Option<Uid>,
    pub title: CompactString,
    pub statement: String,
    pub time_limit: u32,
    pub memory_limit: u32,
}

impl ProblemData {
    pub fn check_unique(&self) -> bool {
        for i in 0..self.files.len() {
            for j in i + 1..self.files.len() {
                if self.files[i].path == self.files[j].path {
                    return false;
                }
            }
        }
        true
    }
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FileChangeEvent {
    SetPriv(CompactString),
    SetPub(CompactString),
    Upload {
        path: CompactString,
        size: u64,
        time: i64,
    },
    Remove(CompactString),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct FileChangeMeta {
    pub evts: Vec<FileChangeEvent>,
}
