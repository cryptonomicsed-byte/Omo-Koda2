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

/// Top-level Habitat: the agent's complete physical-digital presence.
///
/// Holds all areas and resources the agent has registered, plus its canonical
/// address and an optional OmoHome URL for the physical-world integration layer.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Habitat {
    pub habitat_id:   String,
    pub agent_id:     String,
    pub areas:        Vec<Area>,
    pub resources:    Vec<PhysicalResource>,
    pub home_address: HabitatAddress,
    /// URL of the agent's OmoHome (Home Assistant) instance, if any.
    pub omohome_url:  Option<String>,
}

impl Habitat {
    /// Create an empty habitat for the given agent.
    /// Reads `OMOHOME_URL` env var automatically.
    pub fn new(agent_id: &str) -> Self {
        Self {
            habitat_id:  format!("habitat:{agent_id}"),
            agent_id:    agent_id.to_string(),
            areas:       Vec::new(),
            resources:   Vec::new(),
            home_address: HabitatAddress {
                agent_id:    agent_id.to_string(),
                confirmed_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                ..Default::default()
            },
            omohome_url: std::env::var("OMOHOME_URL").ok(),
        }
    }

    /// Register a new area (no-op if area_id already present).
    pub fn register_area(&mut self, area: Area) {
        if !self.areas.iter().any(|a| a.area_id == area.area_id) {
            self.areas.push(area);
        }
    }

    /// Upsert a physical resource (replace if resource_id already present).
    pub fn upsert_resource(&mut self, resource: PhysicalResource) {
        if let Some(existing) = self.resources.iter_mut().find(|r| r.resource_id == resource.resource_id) {
            *existing = resource;
        } else {
            self.resources.push(resource);
        }
    }

    /// All resources within a given area.
    pub fn resources_in(&self, area_id: &str) -> Vec<&PhysicalResource> {
        self.resources.iter().filter(|r| r.area_id == area_id).collect()
    }
}
