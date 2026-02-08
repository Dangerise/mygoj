use super::*;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DownloadToken(Uuid);

impl DownloadToken {
    #[inline]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    #[inline]
    pub fn decode(input: &str) -> Option<Self> {
        Some(Self(Uuid::parse_str(input).ok()?))
    }

    #[inline]
    pub fn encode(&self) -> String {
        self.0.to_string()
    }
}
