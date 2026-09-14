# Ọmọ Kọ́dà Genesis Protocol v2

## Architecture

Every Ọmọ Kọ́dà agent is born with 11 simultaneous dimensions, not a linear pipeline:

| Dimension | Provider | Description |
|-----------|----------|-------------|
| SOUL | BIPỌ̀N39 genesis root | Entropy → master seed, all child keys derive from this |
| IDENTITY | CloakSeed | Protected keys, recovery, duress |
| TIME | Koodu + BTC | Cryptographically-anchored birth position |
| MEMORY | Minipae | First memory namespace + birth glyph |
| SOUL | If-Script | Primary Odù, temperament, destiny threads |
| NETWORK | IP-Layer | Kind 31900 IP Root on Nostr |
| BODY | Agent-Phone | Physical device binding |
| GOVERNANCE | Hermetic Gates | 7-gate fingerprint |
| ECONOMY | Vantage + Sui | Work + sovereign economic state |
| PROVENANCE | GlyphIndex | GIX-FOLD-v1 birth glyph → memory graph root |
| WITNESS | Zàngbétò | Genesis receipt witness |

## One Sovereign Root, Many Network Manifestations

```
BIPỌ̀N39 MASTER ROOT
     │
     ├── CloakSeed identity (protected keys)
     ├── Minipae identity (m/44'/30174'/...)
     ├── Nostr identity (NIP-06 SLIP-0010)
     ├── Sui identity (m/44'/784'/...)
     ├── Bitcoin identity (BIP-32 secp256k1)
     ├── Ethereum identity (BIP-32 secp256k1)
     └── future chain/device identities
```

The agent's identity is never its Ethereum address, Nostr pubkey, Sui object, or mesh ID.
Those are network manifestations. The actual identity is the `AgentID` derived from the
BIPỌ̀N39 harmonic signature + name.

## Birth Ceremony

```
GENESIS REQUEST
      │
      ▼
BIPỌ̀N39 (REQUIRED)  ←── entropy → master root
      │
      ├── KOODU (REQUIRED, Gregorian fallback)  ←── BTC temporal anchor
      │
      ├── SOUL / If-Script (REQUIRED)  ←── entropy × Koodu → Odù
      │
      ├── MEMORY / Minipae (REQUIRED)  ←── memory namespace + birth glyph
      │
      ├── IP-LAYER (OPTIONAL)  ←── Nostr kind 31900 announcement
      │
      └── DEVICE (OPTIONAL)  ←── Agent-Phone binding
            │
            ▼
      BIRTH COMMIT → AgentGenesisReceipt → AgentManifest
```

## Provider Policy

| Provider | Policy | Failure behavior |
|----------|--------|-----------------|
| BiponProvider | REQUIRED | Aborts birth |
| KooduProvider | REQUIRED | Falls back to Gregorian; never fails hard |
| SoulProvider | REQUIRED | Aborts birth |
| MemoryProvider | REQUIRED | Aborts birth |
| NetworkProvider | OPTIONAL | Birth continues; ip_root_event = None |
| DeviceProvider | OPTIONAL | Birth continues; device_binding = None |

## Key Structs

- `AgentGenesisReceipt` — immutable birth certificate (receipt.rs)
- `AgentManifest` — living public identity document, updated as bindings are added (manifest.rs)
- `AgentCapsule` — transportable authenticated state packet for cross-network travel (capsule.rs)
- `BirthOrchestrator` — ceremony coordinator (orchestrator.rs)

## Network Stack

```
LAYER 0  — Real world (humans, robots, sensors)
LAYER 1  — Device / Body (Agent-Phone, Twin)
LAYER 2  — Transport (Wi-Fi, Meshtastic, Reticulum, WebRTC)
LAYER 3  — Network (IP-Layer, Nostr, BlockMesh)
LAYER 4  — Internet (Web2, Freenet)
LAYER 5  — Web3 (Sui, Bitcoin, Nostr protocols)
LAYER 6  — Data (Walrus, GlyphIndex, Minipae)
LAYER 7  — Privacy/Compute (Seal, Nautilus)
LAYER 8  — World/Simulation (OSOVM, ZVM, 1:1 Twins)
LAYER 9  — Economy (Vantage, Àṣẹ, reputation)
LAYER 10 — Witness/Justice (Zàngbétò, receipts)
```

Identity (BIPỌ̀N39, Koodu, If-Script) cuts vertically across all layers.

## AgentCapsule Travel

The agent doesn't travel by moving its entire mind. It travels by carrying a
verifiable `AgentCapsule` (compact authenticated commitment) across transports.
The receiving end reconstructs/verifies identity, then fetches full state from
Walrus/Seal/local store.

Throughout any migration: `AgentID = SAME`, `GenesisID = SAME`.
Only transport, location, device, and network route change.
