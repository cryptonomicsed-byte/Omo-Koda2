pub mod ase;
pub mod bipon39;
pub mod cloak;
pub mod dna;
pub mod duress;
pub mod merkle;
pub mod oauth;
pub mod odu;
pub mod pet;
pub mod safety;
pub mod user;
pub mod vault;
pub mod wallet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AgentId(String);

impl AgentId {
    pub fn new(dna_fingerprint: &str) -> Self {
        Self(format!("agent-{}", &dna_fingerprint[..16]))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        Self(s.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::str::FromStr for AgentId {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
