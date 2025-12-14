use super::*;

const TOKEN_LEN: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Token(pub [u8; TOKEN_LEN]);

impl Token {
    pub fn new() -> Self {
        Self([(); TOKEN_LEN].map(|_| rand::random()))
    }

    pub fn decode(val: &[u8]) -> Option<Self> {
        let bytes = hex::decode(val).ok()?;
        if bytes.len() != TOKEN_LEN {
            return None;
        }
        let mut val = [0; TOKEN_LEN];
        for i in 0..TOKEN_LEN {
            val[i] = bytes[i];
        }
        Some(Self(val))
    }

    pub fn encode(&self) -> String {
        hex::encode(self.0)
    }
}

impl AsRef<[u8]> for Token {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
