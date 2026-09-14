# Agent Habitat — Design Specification
## Status: DESIGN ONLY — No code until reviewed
### Date: 2026-09-13

---

## What This Spec Defines

Two new components only. Everything else already exists.

```
NEW: DIP/crates/dip-adapters/ha/
     Home Assistant ↔ DIP/VCP translation adapter

NEW: Omo-Koda2/src/habitat/
     Agent's physical-world model (Habitat, Area, Room, Zone, Resource, Topology)
```

The rest of the "cut the grass" flow — discovery, job dispatch, waggle coordination,
ARP receipts, Zàngbétò witness, payment — already exists. This spec defines the two
missing pieces that answer WHERE + WHAT before the existing machinery takes over.

---

## Architectural Boundary

```
HUMAN
  │
  │  "Cut the grass"
  ▼
ỌMỌ KỌ́DÀ (intent / policy / consent / capability / job coordination)
  │
  ▼
HABITAT (where is the lawn? what can reach it? is it authorized?)
  │
  ▼
DIP (how do I communicate with the mower?)
  │
  ├── HA adapter (ws://homeassistant.local → HA entity → DeviceManifest)
  ├── Meshtastic adapter (off-grid fallback)
  └── other DIP adapters
  │
  ▼
Physical World
```

Habitat answers: WHERE + WHAT + CAN_REACH
DIP answers: HOW to communicate
VCP answers: WHAT IS this device and how do I bind to its session
Ọmọ Kọ́dà answers: WHY + AM I ALLOWED + how does it fit the organism's goals

---

## Component 1: Omo-Koda2/src/habitat/

### Module structure

```
src/habitat/
├── mod.rs           -- pub re-exports, HabitatError
├── habitat.rs       -- Habitat (root physical context for an agent)
├── area.rs          -- Area (named physical zone within a habitat)
├── resource.rs      -- PhysicalResource (device/robot/sensor/vehicle/presence)
├── topology.rs      -- spatial relationships, reachability graph
└── address.rs       -- HabitatAddress + NetworkRepr extension (network: "habitat")
```

### Type hierarchy

```
Habitat
  │  id: HabitatId            // opaque string, e.g. "home-001"
  │  owner_agent_id: String   // the agent who controls this habitat
  │  display_name: String
  │  areas: Vec<Area>
  │  resources: Vec<PhysicalResource>
  │  topology: Topology
  │  created_at: u64
  │
  ├── Area
  │     id: AreaId
  │     kind: AreaKind        // Indoor | Outdoor | Vehicle | Underground | Aerial
  │     name: String          // "front_lawn", "living_room", "garage"
  │     parent: Option<AreaId>    // containment hierarchy
  │     children: Vec<AreaId>
  │     resources: Vec<ResourceId>
  │     access_constraints: Vec<AccessConstraint>
  │
  ├── PhysicalResource
  │     id: ResourceId
  │     kind: ResourceKind    // Device | Robot | Sensor | Vehicle | HumanPresence | AgentPresence
  │     display_name: String
  │     area_id: Option<AreaId>    // where it currently is (nullable = mobile/unknown)
  │     device_manifest: Option<VcpDeviceManifest>    // filled when VCP session exists
  │     dip_address: Option<NetworkRepr>              // how to reach it
  │     capabilities: Vec<String>                     // ["mow", "navigate", "charge"]
  │     state: ResourceState  // Available | Busy | Offline | Unknown | Charging | Error
  │     battery_pct: Option<u8>
  │     last_seen: Option<u64>
  │
  └── Topology
        reachability: HashMap<(ResourceId, AreaId), Reachability>
        connectivity: HashMap<ResourceId, Vec<Transport>>
```

### Reachability model

Every query before job dispatch checks five conditions:

```
CAN_REACH(resource_id, area_id) -> bool
    // resource is physically capable of reaching the area
    // e.g. mower can reach lawn but not second floor

CAN_OPERATE(resource_id, area_id) -> Result<Vec<String>, ReachabilityError>
    // returns capabilities the resource can actually exercise in that area
    // e.g. mower can "mow" on front_lawn but not "mow" on driveway

IS_AUTHORIZED(agent_id, resource_id) -> bool
    // the agent holds a capability grant for this resource
    // checked against CapabilityRegistry (existing)

IS_AVAILABLE(resource_id) -> bool
    // resource.state == Available && battery_pct above threshold

IS_SAFE(area_id, operation: &str) -> Result<(), SafetyError>
    // area constraints (gate closed? weather? children present?)
    // pluggable: default=Ok, HA adapter fills real constraints

Reachability {
    can_reach: bool,
    transport_options: Vec<TransportOption>,  // WiFi, Meshtastic, Bluetooth, etc.
    estimated_minutes: Option<u32>,
    constraints: Vec<AccessConstraint>,
}

TransportOption {
    kind: TransportKind,    // reuse VCP's TransportKind + add Meshtastic, BLE
    adapter_id: String,     // which DIP adapter handles this
    quality: u8,            // 0-100 estimated reliability
}
```

### Physical addressing

Extends DIP's `NetworkRepr` — no new UURI standard:

```rust
// In DIP NetworkRepr (existing):
// pub struct NetworkRepr {
//     pub network: String,   // "nostr" | "a2a" | "mesh" | "mcp" | ...
//     pub address: String,
//     pub public_key: Option<String>,
// }

// New usage — network = "habitat":
NetworkRepr {
    network: "habitat",
    address: "home-001/area/front_lawn",   // habitat_id/area/area_id
    public_key: None,
}

// For a specific resource in a habitat:
NetworkRepr {
    network: "habitat",
    address: "home-001/resource/mower-77",
    public_key: None,  // or device's Ed25519 pubkey if it has one
}
```

This slots into `AgentManifest.network.*` and `DipIdentity.networks` with zero changes.

### Habitat lifecycle

```
UNREGISTERED
    │  register(habitat_config)
    ▼
REGISTERED
    │  attach_adapter(dip_adapter_id)
    ▼
CONNECTED
    │  (HA sends initial state dump or adapter discovers devices)
    ▼
POPULATED     -- areas, resources, topology known
    │  sync_state() called periodically / on event
    ▼
LIVE          -- reachability queries available
    │  agent offline
    ▼
SUSPENDED     -- snapshot sealed, resume on reconnect
```

---

## Component 2: DIP/crates/dip-adapters/ha/

### What it does

Bridges HA's WebSocket API to DIP's `DipEnvelope` format and populates
`VcpDeviceManifest` for each HA entity the agent cares about.

Does NOT import HA Python code. HA runs as a separate process. This adapter
is a pure Rust WebSocket client speaking HA's documented JSON protocol.

### Module structure

```
DIP/crates/dip-adapters/ha/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── client.rs        -- HA WebSocket client (connect, auth, subscribe)
│   ├── entity.rs        -- HA entity → VcpDeviceManifest mapping
│   ├── envelope.rs      -- HA event → DipEnvelope conversion
│   ├── service.rs       -- DipEnvelope capability call → HA service call
│   └── config.rs        -- HaConfig (url, token, entity_filter)
```

### Interface definition

```
HaAdapter implements DipAdapter (existing EcosystemAdapter trait):
    fn adapter_id() -> &str { "home-assistant" }
    fn adapter_kind() -> AdapterKind { AdapterKind::HomeAssistant }  // new variant

Connection:
    async fn connect(config: HaConfig) -> Result<Self, HaError>
        // Opens ws://[host]:[port]/api/websocket
        // Sends auth message with long-lived access token
        // Subscribes to state_changed events

Inbound (HA → DIP):
    async fn poll_event() -> Option<DipEnvelope>
        // Receives state_changed event from HA
        // Maps entity_id + new_state → DipEnvelope{
        //     from: NetworkRepr { network: "habitat", address: "ha:[entity_id]" },
        //     payload: DipMessage::EventBroadcast { ... }
        // }

    fn entity_to_device_manifest(entity: &HaEntity) -> VcpDeviceManifest
        // Maps HA domain to VCP DeviceClass:
        //   "lawn_mower"  → DeviceClass::GroundRobot
        //   "vacuum"      → DeviceClass::GroundRobot
        //   "climate"     → DeviceClass::SensorNode
        //   "light"       → DeviceClass::SensorNode
        //   "lock"        → DeviceClass::SensorNode
        //   "camera"      → DeviceClass::SensorNode
        //   "cover"       → DeviceClass::SensorNode (garage doors, blinds)
        //   "sensor"      → DeviceClass::SensorNode
        //   "binary_sensor" → DeviceClass::SensorNode
        //   default       → DeviceClass::EdgeServer
        // Maps HA capabilities → VCP ActuatorSpec / SensorSpec
        // Maps HA state → ResourceState

Outbound (DIP → HA):
    async fn send_envelope(envelope: DipEnvelope) -> Result<RouteReceipt, HaError>
        // DipEnvelope{payload: DipMessage::ToolCall { tool, params }} →
        // POST /api/services/[domain]/[service]  (HA REST API)
        // OR ws call_service message
        // Returns RouteReceipt with ha_context_id

State query:
    async fn get_entity_state(entity_id: &str) -> Result<HaEntity, HaError>
        // GET /api/states/[entity_id]
        // Used to populate Habitat topology at registration time
        // and to re-check IS_AVAILABLE before job dispatch

    async fn list_entities(domain_filter: Option<&str>) -> Result<Vec<HaEntity>, HaError>
        // GET /api/states (filtered)
        // Used at habitat REGISTERED→POPULATED transition
```

### HaConfig

```rust
HaConfig {
    url: String,                    // "ws://homeassistant.local:8123"
    access_token: String,           // long-lived access token (stored in Tier 1 identity vault)
    entity_filter: Vec<String>,     // domains to care about, e.g. ["lawn_mower", "lock", "camera"]
    area_mapping: HashMap<String, AreaId>,  // HA area_id → our AreaId
    polling_interval_secs: u32,     // state refresh cadence (default: 30)
}
```

### DipMessage mapping

```
HA state_changed →  DipMessage::EventBroadcast {
    kind: "ha.state_changed",
    entity_id,
    old_state,
    new_state,
    attributes,
    tags: [("ha_domain", domain), ("area", area_id)]
}

HA call_service result → DipMessage::ToolResult {
    tool: "[domain].[service]",
    result: serde_json::Value,
    success: bool,
}

Agent capability call → DipMessage::ToolCall {
    tool: "[domain].[service]",    // e.g. "lawn_mower.start_mowing"
    params: { "entity_id": "...", ... }
}
```

---

## The Lawn-Mowing Flow (End to End)

Using only existing + newly specified components:

```
1. HUMAN INTENT
   "Cut the grass"
   │
   ▼
2. ỌMỌ KỌ́DÀ — intent parsing (If-Script or interpreter)
   Produces: Task { kind: "mow", target: "front_lawn", constraints: { window, price } }
   │
   ▼
3. HABITAT — spatial resolution
   habitat.resolve("front_lawn") → Area { id: "front_lawn", ... }
   habitat.topology.reachable_resources("front_lawn", "mow")
       → [ResourceId("mower-77"), ResourceId("mower-99")]
   habitat.check_all(agent_id, "mower-77", "front_lawn", "mow")
       → CAN_REACH=true, CAN_OPERATE=["mow"], IS_AUTHORIZED=true,
          IS_AVAILABLE=true, IS_SAFE=true
   │
   ▼
4. DISCOVERY (existing: Vantage find_capable_agents())
   Also queries agent mesh for external providers (landscaper agents)
   Returns: [mower-77 (local, $0), landscaper-12 (external, $30)]
   │
   ▼
5. POLICY + SELECTION (existing: CapabilityRegistry + CapabilityFabric)
   Agent applies owner policy → selects mower-77 (local, free)
   │
   ▼
6. JOB (existing: WorkKind::Job in coordination/work_ref.rs)
   Creates: Job { id: "job-000183", resource: "mower-77", area: "front_lawn",
                  operation: "mow", authorized_by: agent_id }
   │
   ▼
7. CLAIM (existing: waggle FieldVerb::Claim)
   Waggle claims the job, locks resource
   │
   ▼
8. EXECUTE via DIP
   DIP routes to HomeAssistant adapter
   HaAdapter.send_envelope(DipEnvelope{ ToolCall: "lawn_mower.start_mowing" })
   HA executes on physical mower
   │
   ▼
9. WITNESS (existing: Zàngbétò)
   HaAdapter streams state_changed events → DipEnvelope::EventBroadcast
   Zàngbétò records observation bundle
   │
   ▼
10. COMPLETION (existing: waggle FieldVerb::Mark + Release)
    HA entity state → "docked" → adapter emits completion event
    Waggle marks complete, releases resource
    │
    ▼
11. RECEIPT (existing: ARP ActionReceipt)
    ArpBridge.act_receipt(agent_id, "lawn_mower.start_mowing", ...)
    Sealed in ARP receipt chain
    │
    ▼
12. SETTLEMENT (existing: Vantage economy, $0 for local job)
    If external agent: Sui payment via Vantage
```

No new code in steps 4–12. Only steps 3 and 8 use the new habitat/ha components.

---

## Security Boundaries

```
Boundary 1: HA access token
    Stored in: Tier 1 identity vault (identity_vault sealed blob)
    Never passed to: adapters as plaintext
    Adapter receives: derived session token OR token is loaded at adapter-init time
    and stays inside the adapter process — NOT in DipEnvelope

Boundary 2: Habitat sovereignty
    Only the owner agent can modify habitat topology
    Other agents can QUERY reachability (with permission) but cannot register/remove resources
    Habitat data stored in: Tier 2 memory (memory_vault, local sealed blob)
    NOT broadcast to relay (no minipae write for habitat topology — this is local private data)

Boundary 3: Physical authorization
    IS_AUTHORIZED check happens in Omo-Koda2 (CapabilityRegistry)
    before ANY DipEnvelope is sent to the HA adapter
    HA adapter trusts the caller — it does NOT re-check authorization
    Consequence: HA adapter is a privileged component — must only be reachable by Omo-Koda2

Boundary 4: HA token scope
    HA long-lived tokens are scoped to HA user accounts, not agent identities
    The agent should use a dedicated HA user (not the owner's personal HA account)
    Token stored in Tier 1 — never appears in logs, receipts, or DipEnvelopes

Boundary 5: Meshtastic fallback
    When internet/WiFi down, DIP routes through Meshtastic adapter instead
    HA adapter is unavailable (HA requires local network)
    Fallback: direct Meshtastic commands to devices that speak it natively
    Agent must degrade gracefully when HA is unreachable
```

---

## Testable Acceptance Criteria

These are the conditions that confirm the implementation is correct:

### habitat/ module
- [ ] `Habitat::register(config)` populates areas and resources from a static config
- [ ] `topology.can_reach("mower-77", "front_lawn")` returns `true` when configured
- [ ] `topology.can_reach("mower-77", "second_floor")` returns `false`
- [ ] `IS_AVAILABLE("mower-77")` returns `false` when `resource.state == Busy`
- [ ] `HabitatAddress` serializes to `NetworkRepr { network: "habitat", address: "..." }`
- [ ] `HabitatAddress` round-trips through DIP `AgentManifest.network`

### ha/ adapter
- [ ] Adapter connects to HA WebSocket and authenticates with `HaConfig.access_token`
- [ ] `list_entities("lawn_mower")` returns entities with `device_manifest.device_class == GroundRobot`
- [ ] `poll_event()` returns `DipEnvelope::EventBroadcast` on HA `state_changed` event
- [ ] `send_envelope(ToolCall{ "lawn_mower.start_mowing" })` results in HA service call
- [ ] Adapter returns `HaError::Unauthorized` (not panic) on invalid token
- [ ] Adapter returns `HaError::Unavailable` (not panic) when HA is unreachable

### integration (habitat + ha adapter)
- [ ] Habitat populated from HA entity list at `REGISTERED→POPULATED` transition
- [ ] HA `state_changed` event updates `resource.state` in Habitat
- [ ] Full lawn-mow flow executes in test harness with mock HA WebSocket server
- [ ] Job completes and ARP `ActionReceipt` is produced
- [ ] HA unreachable → agent falls back to Meshtastic, does NOT crash

---

## What NOT to Build

| Proposed | Reason |
|---|---|
| HA entity model inside Omo-Koda2 | That's the adapter's job — don't duplicate |
| New job marketplace | WorkKind::Job + waggle already handle it |
| New state machine | SovereignEventBus already handles state |
| New discovery registry | Vantage find_capable_agents() already exists |
| New UURI standard | network: "habitat" in DIP NetworkRepr is sufficient |
| HA Python code in Rust | HA runs as separate process — WebSocket bridge only |
| Habitat data on minipae relay | Physical topology is local private data (Tier 2 local only) |
| Competing auth system | HA tokens for HA devices; VCP grants for agent-to-device sessions — both coexist |
| New area/room types in VCP | VCP models device sessions, not spaces |

---

## Implementation Order (after this spec is reviewed)

```
Phase 1 — Habitat types (no I/O, pure data model):
    src/habitat/habitat.rs
    src/habitat/area.rs
    src/habitat/resource.rs
    src/habitat/address.rs  (HabitatAddress + NetworkRepr usage)
    src/habitat/mod.rs

Phase 2 — Topology + reachability:
    src/habitat/topology.rs
    Reachability queries: CAN_REACH, CAN_OPERATE, IS_AVAILABLE, IS_SAFE
    (IS_AUTHORIZED delegates to existing CapabilityRegistry)

Phase 3 — HA adapter skeleton (no live connection yet):
    DIP/crates/dip-adapters/ha/src/config.rs
    DIP/crates/dip-adapters/ha/src/entity.rs  (HA entity → DeviceManifest)
    DIP/crates/dip-adapters/ha/src/envelope.rs  (HA event → DipEnvelope)
    AdapterKind::HomeAssistant variant added to dip-types

Phase 4 — Live HA connection:
    DIP/crates/dip-adapters/ha/src/client.rs  (WebSocket, auth, subscribe)
    DIP/crates/dip-adapters/ha/src/service.rs  (service calls)
    Integration: HaAdapter populates Habitat on connect

Phase 5 — Full lawn-mow flow test:
    Mock HA WebSocket server in tests
    Full job dispatch → execute → receipt path
```

---

## Open Questions

None blocked. Architecture is locked. This spec is ready for Phase 1 implementation.
If a question arises during Phase 1, it will be flagged before writing code.
