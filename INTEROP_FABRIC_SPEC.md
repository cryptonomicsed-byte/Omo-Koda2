# Ọmọ Kọ́dà 2 — Universal Interoperability Fabric
## Design Specification (No Code — Interfaces Only)
### Date: 2026-09-13

---

## The Core Architectural Finding

Before designing anything new, the cross-repo audit produced this critical result:

**The interop fabric already exists. It is distributed across the sovereign protocol stack.**

| Proposed Layer | Already Exists In | Key Type |
|---------------|------------------|----------|
| UURI (resource naming) | **DIP** | `NetworkRepr` — `nostr:`, `a2a:`, `mesh:`, `mcp:`, `p2p:`, `free:`, `http:` prefixes |
| UAP Envelope (wire protocol) | **DIP** + **ARP** | `DipEnvelope` (transport) + `ActionReceipt` (canonical receipt) |
| Event Normalization | **Vantage** + **DIP** | `VantageEvent` bus + `EventBroadcast` in DipEnvelope |
| Discovery | **Vantage** + **DIP** + **UCX** | `CapabilityRegistry.find_capable_agents()`, `AdapterRegistry`, `VantageDiscovery` |
| A2A Capability Exchange | **VCP** | `HandshakeEngine` — 7-step Challenge/Auth/CapNeg/Grant/Revoke |

**Consequence: Ọmọ Kọ́dà 2 does NOT build these layers from scratch.**

Instead, Ọmọ Kọ́dà 2 becomes a CLIENT of these sovereign protocols.

The missing piece is a **Bridge Layer** inside Ọmọ Kọ́dà 2 that connects its existing
identity/governance/memory systems to the protocols that already exist across the stack.

---

## Architectural Boundary Clarification

```
                    SOVEREIGN PROTOCOL STACK
                    (already built, stays sovereign)
                    ┌─────────────────────────────────┐
                    │  DIP   VCP   ARP   Vantage  UCX │
                    └──────────────┬──────────────────┘
                                   │ bridge (to be designed)
                    ┌──────────────▼──────────────────┐
                    │         ỌMỌ KỌ́DÀ 2              │
                    │                                  │
                    │  Identity  Governance  Memory    │
                    │  (already built — stays intact)  │
                    └──────────────────────────────────┘
```

Ọmọ Kọ́dà 2 does NOT absorb DIP, VCP, ARP, or Vantage.
Each protocol remains sovereign. Ọmọ Kọ́dà 2 speaks their wire formats.

---

## What Already Exists vs. What Bridges Are Needed

### UURI — No New Code Needed

DIP already defines `NetworkRepr`:
```
network: "nostr" | "a2a" | "mesh" | "mcp" | "p2p" | "free" | "http"
address: <ecosystem-specific address>
public_key: Option<String>
```

And `DipIdentity` is a canonical DID with `networks: Vec<NetworkRepr>` — one identity,
many ecosystem projections.

**Decision:** Ọmọ Kọ́dà 2 adopts DIP's `NetworkRepr` as its resource addressing scheme.
The `NetworkSection` in `AgentManifest` maps directly to `DipIdentity.networks`.
No new UURI type needed inside Ọmọ Kọ́dà 2.

**Bridge needed:** A function that converts `AgentManifest.network` → `DipIdentity`.

---

### UAP Envelope — Two Existing Types, One Composition

**For outbound actions** (Ọmọ Kọ́dà → ecosystem):
DIP's `DipEnvelope` carries: from/to identity, payload (`DipMessage` enum), signature.
`DipMessage` variants: `AgentDelegate`, `ToolCall`, `ToolResult`, `CapabilityAd`,
`IdentityClaim`, `EventBroadcast`, `ReceiptRelay`.

**For receipts** (canonical proof of any consequential action):
ARP's `ActionReceipt` carries the full Principal→Capability→Action→Evidence→Receipt chain.
`ReceiptKind` enum includes `AgentLifecycle`, `Economic`, `Governance`, `MeshEvent`, etc.

**The composition:**
```
Intent (If-Script / interpreter)
    │
    ▼
ARP ActionReceipt (canonical proof — built before execution)
    │
    ▼
DipEnvelope (transport wrapper — built at send time)
    │
    ▼
network_router.rs Transport (physical delivery)
```

`network_router.rs` stays — it IS the physical delivery layer.
`DipEnvelope` sits ABOVE it as the semantic wrapper.
`ActionReceipt` is the permanent record, independent of transport.

**Bridge needed:** `DipBridge` — converts Ọmọ Kọ́dà's internal action + identity
into a `DipEnvelope` for outbound delivery via `network_router.rs`.

---

### Event Normalization — Vantage Is the Hub

Vantage already has `VantageEvent` with:
- `event_type`, `actor_id`, `aggregate_id`, `payload`, `signature`
- `principal_id → session_id → execution_id → receipt_id` chain (P0–P4)

DIP has `EventBroadcast` variant in `DipEnvelope` with Nostr-compatible tags.

**Decision:** Ọmọ Kọ́dà 2's `SovereignEventBus` subscribes to Vantage's event stream
for external ecosystem events. DIP's `EventBroadcast` feeds into the Vantage bus.

The normalization path is:
```
Nostr event / Meshtastic packet / Web3 tx / Web2 webhook / Freenet update
    │
    ▼
DIP adapter (translates native protocol → DipEnvelope.EventBroadcast)
    │
    ▼
Vantage VantageEvent bus
    │
    ▼
Ọmọ Kọ́dà 2 SovereignEventBus (receives normalized VantageEvents)
    │
    ▼
If-Script / intent engine
```

**Bridge needed:** A `VantageEventSubscriber` inside Ọmọ Kọ́dà 2 that connects
Vantage's WebSocket/HTTP event stream to `SovereignEventBus`.

---

### Discovery — Vantage Already Has It

`Vantage.CapabilityRegistry.find_capable_agents(required_capabilities)` returns agents
sorted by capability match count + reputation.

`UCX.VantageDiscovery` fetches `Vec<ProviderCapability>` from Vantage.

`DIP.AdapterRegistry` indexes which adapters support which capability kinds.

**Decision:** Ọmọ Kọ́dà 2 registers itself with Vantage's `CapabilityRegistry` at birth.
Discovery queries route through Vantage. No new discovery registry inside Ọmọ Kọ́dà 2.

**Bridge needed:** A `VantageRegistration` call during `birth()` that registers the agent's
capabilities (from `CapabilityRegistry.issue_birth_grants()` output) with Vantage.

---

### A2A Capability Exchange — VCP Has the Sophisticated Protocol

VCP's `HandshakeEngine` implements the complete 7-step negotiation:
1. Discovery (device/agent self-description)
2. Challenge (nonce + required capabilities)
3. Auth (Ed25519 proof of key ownership)
4. CapNeg (offered capability subset)
5. Grant (scoped, expiring authorization with safety class)
6. Revoke (any party, any time)
7. VcpReceipt (session audit)

**Key distinction between VCP `CapabilityGrant` and Omo-Koda2 `CapabilityGrant`:**
- **VCP grant** = session-scoped device occupancy, safety-class-aware
- **Omo-Koda2 grant** = ecosystem-scoped identity flag (READ/WRITE/SIGN/PAY etc.)
- These are DISTINCT, not duplicates. An agent holds both simultaneously.
- Integration: VCP's `Challenge.required_capabilities` can reference Omo-Koda2 scope flags.

**Decision:** Ọmọ Kọ́dà 2 uses VCP's handshake for device capability exchange.
For agent-to-agent negotiation (no device involved), the existing `a2a/delegation.rs`
is extended to carry DIP identity proofs — NOT a new A2A protocol.

**Bridge needed:** A `VcpClient` inside Ọmọ Kọ́dà 2 that initiates VCP handshakes
for device binding, using the agent's `AgentManifest` as the `DeviceManifest` equivalent.

---

## The Five Bridges — Interface Definitions (No Implementation)

These are the only new things Ọmọ Kọ́dà 2 needs. All are thin adapters.

### Bridge 1: DipBridge
**File:** `src/bridge/dip.rs` (the `bridge` module already exists in `lib.rs`)
**Direction:** Outbound — Ọmọ Kọ́dà 2 → DIP

```
Interface DipBridge:
    fn agent_to_dip_identity(manifest: &AgentManifest) -> DipIdentity
    fn wrap_action(from: &AgentManifest, to_agent_id: &str, msg: DipMessage) -> DipEnvelope
    fn send_via_router(envelope: DipEnvelope, router: &NetworkRouter) -> RouteReceipt
    fn identity_claim(manifest: &AgentManifest) -> DipEnvelope  // IdentityClaim variant
```

`DipIdentity` is constructed from `AgentManifest.network.*` fields.
`DipEnvelope.from` = agent's DipIdentity.
`DipEnvelope.signature` = sign with agent's Ed25519 key.

---

### Bridge 2: ArpBridge
**File:** `src/bridge/arp.rs`
**Direction:** Both — creates ARP receipts from Ọmọ Kọ́dà actions; reads incoming receipts

```
Interface ArpBridge:
    fn birth_receipt(genesis: &AgentGenesisReceipt) -> ActionReceipt  // ReceiptKind::AgentLifecycle
    fn think_receipt(agent_id, thought_hash, koodu, hermetic_score) -> ActionReceipt
    fn act_receipt(agent_id, tool, params, result_hash, koodu) -> ActionReceipt  // ReceiptKind::Compute
    fn economic_receipt(agent_id, action, amount, currency, counterparty) -> ActionReceipt  // ReceiptKind::Economic
    fn from_act_receipt(act: &Receipt) -> ActionReceipt  // convert existing receipt to ARP format
```

Existing `receipt/` chain stays intact. `ArpBridge` wraps it for cross-repo compatibility.
Does NOT replace `ActReceipt` — runs alongside it.

---

### Bridge 3: VantageEventSubscriber
**File:** `src/bridge/vantage_events.rs`
**Direction:** Inbound — Vantage → Ọmọ Kọ́dà 2 SovereignEventBus

```
Interface VantageEventSubscriber:
    fn connect(vantage_url: &str, agent_id: &str) -> Result<Self, BridgeError>
    fn normalize(vantage_event: VantageEvent) -> Option<SovereignEvent>  // None = drop irrelevant events
    async fn run(self, bus: &SovereignEventBus)  // pump loop
```

`normalize()` maps `VantageEvent.event_type` to existing `SovereignEvent` variants.
Unknown event types are dropped (fail-open).
Does NOT change `SovereignEventBus` — purely additive.

---

### Bridge 4: VantageRegistration
**File:** `src/bridge/vantage_reg.rs`
**Direction:** Outbound — Ọmọ Kọ́dà 2 → Vantage capability registry

```
Interface VantageRegistration:
    async fn register_at_birth(
        genesis: &AgentGenesisReceipt,
        capability_grants: &[CapabilityGrant],
        vantage_url: &str,
    ) -> Result<(), BridgeError>

    async fn update_capabilities(
        agent_id: &str,
        grants: &[CapabilityGrant],
        vantage_url: &str,
    ) -> Result<(), BridgeError>
```

Called from `birth()` after `CapabilityRegistry.issue_birth_grants()`.
Converts `CapabilityGrant.scopes` to Vantage's capability string format.
Fire-and-forget (fail-open — if Vantage is unreachable, birth continues).

---

### Bridge 5: VcpClient (stub only — no implementation yet)
**File:** `src/bridge/vcp.rs`
**Direction:** Both — Ọmọ Kọ́dà 2 ↔ VCP broker

```
Interface VcpClient:
    async fn initiate_handshake(
        agent_manifest: &AgentManifest,
        device_id: &str,
        broker_url: &str,
    ) -> Result<VcpReceipt, BridgeError>

    async fn revoke_session(session_id: &str, broker_url: &str) -> Result<(), BridgeError>

    fn grant_to_device_binding(grant: &vcp::CapabilityGrant) -> DeviceBinding  // for genesis receipt
```

Stub only in this phase. Full implementation requires VCP broker to be reachable.
`DeviceBinding` in `AgentGenesisReceipt` is populated when a VCP grant is issued.

---

## What NOT to Build

Based on both audits, these should NOT be added to Ọmọ Kọ́dà 2:

| Proposed | Reason Not To Build |
|---------|---------------------|
| New UURI type in genesis/ | DIP's `NetworkRepr` is already the standard |
| Event normalizer inside Ọmọ Kọ́dà 2 | Vantage already does this — bridge to it |
| Discovery registry | Vantage has `find_capable_agents()` — register there |
| A2A handshake protocol | VCP already has the sophisticated version |
| New credential vault abstraction | Existing `identity/{vault,wallet,oauth}` has separate security boundaries for a reason — do NOT merge yet |
| Agent Constitution document | `steward/constitution.rs` already encodes this as executable law |

---

## Remaining Genuine Gaps (Not Blocked By Protocol Stack)

These DO need to be built, but are independent of the interop fabric:

### Gap A: Lifecycle Completion
Missing states: MIGRATION, HIBERNATION, RETIREMENT + cryptographic revocation.
- RETIREMENT emits an ARP `ActionReceipt` with `ReceiptKind::AgentLifecycle` and a `revocation_key`.
- HIBERNATION = snapshot + seal + pause SovereignEventBus.
- MIGRATION = seal capsule + send via DipBridge + resume on new host.
- No code yet — design only here.

### Gap B: Agent Forking
New territory. Requires separate design session. Key open questions:
- What does the child get from the parent BIPỌ̀N39 seed? (recommendation: derive independently, parent signs birth)
- Memory inheritance: None / Selective / Copy — user must decide.
- Reputation inheritance: None / Derived — user must decide.
- Constitution inheritance: always inherit, child may amend.

### Gap C: Credential Unification Index
Not a vault merger — just an INDEX.
`CredentialIndex` maps `(agent_id, credential_kind, ecosystem) → location` without merging
the actual secret stores. OAuth in `identity/oauth.rs`, keys in `identity/vault.rs`, etc. stay separate.
The index allows `VantageRegistration` and `DipBridge` to look up which credential to use.

---

## Implementation Order (No Code Until Reviewed)

```
Phase 1 (bridge foundations):
    DipBridge.agent_to_dip_identity()
    ArpBridge.birth_receipt()
    VantageRegistration.register_at_birth()

Phase 2 (live interop):
    VantageEventSubscriber (inbound events)
    DipBridge.wrap_action() + send_via_router()
    ArpBridge.act_receipt()

Phase 3 (lifecycle):
    RETIREMENT + revocation
    HIBERNATION + resume
    MIGRATION via DipBridge + AgentCapsule

Phase 4 (after design session):
    Agent Forking / Genealogy
    VcpClient full implementation
    CredentialIndex
```

---

## Questions That Need Your Decision Before Phase 1

1. **DipIdentity as the canonical external identity**: Agree to use DIP's `NetworkRepr` as the standard? 
   Or does Ọmọ Kọ́dà 2 maintain its own addressing scheme and translate at the bridge boundary?

2. **ARP receipts alongside existing receipts**: The existing `ActReceipt` chain stays.
   `ArpBridge` produces a SECOND receipt in ARP format for cross-repo compatibility.
   Is that double-receipting acceptable, or should one replace the other over time?

3. **Vantage registration at birth**: If Vantage is unreachable at birth time (offline birth),
   registration is deferred. When should it retry? On first network contact? Manually?

4. **Agent forking inheritance**: Memory, reputation, and constitution inheritance rules.
   These must be defined before any forking code can be written.
