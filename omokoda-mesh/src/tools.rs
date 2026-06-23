//! Mesh tool descriptors.
//!
//! These are plain data structures describing the 9 mesh tools. The actual
//! omokoda-core ToolRegistry registers them as `Box<dyn Tool>` using adapters
//! defined in omokoda-core's mesh_tools.rs shim — this keeps omokoda-mesh free
//! of a dependency on omokoda-core.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshToolInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub required_tier: u8,
    pub is_write: bool,
}

pub const MESH_TOOLS: &[MeshToolInfo] = &[
    MeshToolInfo {
        name: "mesh_propose",
        description: "Propose a commitment exchange with a neighbor agent. Params: {neighbor, give:[{kind,description}], take:[{kind,description}], duration_secs}",
        required_tier: 2,
        is_write: true,
    },
    MeshToolInfo {
        name: "mesh_respond",
        description: "Accept, reject, or counter a received proposal. Params: {negotiation_id, decision: \"accept\"|\"reject\"|\"counter\"}",
        required_tier: 2,
        is_write: true,
    },
    MeshToolInfo {
        name: "mesh_query_resources",
        description: "List available shared resources on the block. Params: {block_id?, filter?}",
        required_tier: 1,
        is_write: false,
    },
    MeshToolInfo {
        name: "mesh_reserve_resource",
        description: "Reserve a shared block resource for a duration. Params: {resource_id, duration_secs, purpose}",
        required_tier: 2,
        is_write: true,
    },
    MeshToolInfo {
        name: "mesh_release_resource",
        description: "Release a previously reserved resource. Params: {resource_id}",
        required_tier: 2,
        is_write: true,
    },
    MeshToolInfo {
        name: "mesh_query_neighbors",
        description: "List known neighbor agents on the block with their roles and trust scores. Params: {block_id?, filter?}",
        required_tier: 1,
        is_write: false,
    },
    MeshToolInfo {
        name: "mesh_query_trust",
        description: "Get the trust score and commitment history for a neighbor. Params: {agent_id}",
        required_tier: 1,
        is_write: false,
    },
    MeshToolInfo {
        name: "mesh_signal_event",
        description: "Broadcast an event to all agents on the block. Params: {event_type, details}",
        required_tier: 2,
        is_write: true,
    },
    MeshToolInfo {
        name: "mesh_discover_capabilities",
        description: "Fetch capability cards from one or all neighbors. Params: {agent_id?}",
        required_tier: 1,
        is_write: false,
    },
];
