//! Habitat Phase 1 — physical + digital presence types.
//!
//! Describes where an agent IS and what resources surround it, so the
//! cognitive loop can reason about physical availability, power, bandwidth,
//! and adjacent peers without coupling to any specific sensor API.

use serde::{Deserialize, Serialize};

/// A named physical or logical area the agent occupies or monitors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Area {
    /// Stable identifier — e.g. "node-room-a", "home-office", "field-site-3".
    pub area_id:     String,
    /// Human-readable label.
    pub label:       String,
    /// Optional GPS bounding box [lat_min, lon_min, lat_max, lon_max].
    pub bbox:        Option<[f64; 4]>,
    /// Floor / floor-plan identifier for indoor spaces.
    pub floor:       Option<String>,
    /// Custom metadata (building ID, provider region, zone type, etc.)
    pub meta:        serde_json::Value,
}

/// A physical or virtual resource that the agent can sense or control.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalResource {
    /// Stable identifier.
    pub resource_id:   String,
    /// Type tag — "sensor", "actuator", "network", "power", "storage", "peer".
    pub kind:          String,
    /// Area this resource belongs to.
    pub area_id:       String,
    /// Human-readable label.
    pub label:         String,
    /// Last known value (unit is resource-type specific).
    pub last_value:    Option<serde_json::Value>,
    /// Unix seconds of last update.
    pub last_seen_at:  Option<u64>,
}

/// Composite habitat address — locates an agent in the physical-digital space.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HabitatAddress {
    /// Owning agent's id.
    pub agent_id:     String,
    /// Current primary area.
    pub area_id:      Option<String>,
    /// GPS coordinates (lat, lon) if available.
    pub gps:          Option<(f64, f64)>,
    /// IP address of the local device.
    pub ip:           Option<String>,
    /// Meshtastic node address if on-mesh.
    pub mesh_node:    Option<String>,
    /// Nostr npub for this agent's public Nostr presence.
    pub nostr_npub:   Option<String>,
    /// Unix seconds — when this address was last confirmed.
    pub confirmed_at: u64,
}
