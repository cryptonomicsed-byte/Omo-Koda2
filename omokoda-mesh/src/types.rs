use serde::{Deserialize, Serialize};

pub type BlockId = String;
pub type ResourceId = String;
pub type AgentId = String;
pub type NegotiationId = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeshRole {
    Home,
    Business,
    Resource,
    Pattern,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeshMembership {
    Probation,
    Active,
    Suspended,
}

impl MeshMembership {
    pub fn as_u32(&self) -> u32 {
        match self {
            MeshMembership::Probation => 0,
            MeshMembership::Active => 1,
            MeshMembership::Suspended => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeoCoord {
    pub lat: f64,
    pub lon: f64,
}
