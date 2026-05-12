use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OduSeed(pub [u8; 32]);

impl OduSeed {
    pub fn new(seed: [u8; 32]) -> Self {
        Self(seed)
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn len(&self) -> usize {
        32
    }
}

impl AsRef<[u8]> for OduSeed {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OduIdentity {
    pub primary_index: u8,
    pub mnemonic: String,
}
