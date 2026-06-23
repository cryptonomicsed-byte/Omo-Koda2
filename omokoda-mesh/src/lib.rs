//! Ọmọ Kọ́dà Block Mesh
//!
//! Spatial multi-agent topology layer. Household, business, and resource agents
//! on a physical block discover each other, negotiate commitments, and coordinate
//! through the existing birth/think/act primitives and receipt chain.

pub mod negotiation;
pub mod resource;
pub mod router;
pub mod state;
pub mod tools;
pub mod trust;
pub mod types;

pub use router::MeshRouter;
pub use state::{CapabilityCard, MeshState, NeighborProfile, ResourceEntry};
pub use tools::MESH_TOOLS;
pub use trust::{MeshTrustModel, NeighborTrust, ProbationManager};
pub use types::{AgentId, BlockId, MeshMembership, MeshRole, NegotiationId, ResourceId};
