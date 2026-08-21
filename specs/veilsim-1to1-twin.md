# VeilSim 1:1 Real-World Twin (v1.0)

**Status: locked design, not implemented.** First locked 2026-07-12 (posim-reference-architecture.md
§30, Zelda tile-world), consolidated 2026-07-17 (vault note
`Memory/REM/1:1-World/Zelda-Tile/GLM-256 Consolidated Sweep`), extended 2026-08-21 with the
concrete capture/attestation stack. Depends on task #6 (cross-machine determinism) closing first —
this cannot start before VeilSim is provably reproducible.

## What this is not

Two separate "1:1" concepts exist in this ecosystem and must not be conflated:

- **`WorldGenerator.jl`** — per-agent *simulated* training worlds (MuJoCo/Dojo procedural
  terrain+physics, seeded, deterministic). Pure sketch, unimplemented. Not this doc.
- **This doc** — a *real-world* geospatial digital twin agents train against and later act in.

## The locked shape

Open-source world maps are the base cache; devices fill only the delta on top of that cache;
capture is stored in Walrus blobs; each blob is a mineable PoSim job; twin data gets sold over
time to fund the DePIN flywheel back through the Èṣù 3.69% router. `dimos` was chosen
specifically for building this real-world twin (distinct from `WorldGenerator.jl`'s simulated
worlds, which export trained policy *to* dimos for real hardware).

## Reconstruction method: Gaussian splatting

Any device that can capture images/video of its surroundings contributes source frames;
3D Gaussian splatting reconstructs the local scene from those frames into the twin's delta layer
over the base cache. Device-agnostic by construction — the input is just images, not a
particular sensor stack, so a phone camera, a drone gimbal, or a dashcam all feed the same
pipeline.

## Capture sources — any device mining the network

Not a closed sensor list. Any DePIN network with cameras/positioning already deployed can
contribute delta capture: Hivemapper (dashcam mapping, closest fit — already does street-level
imagery), DIMO (vehicle telemetry/position), Helium (coverage/positioning, weaker fit for
imagery specifically but real for presence/location proof). None of these are wired in yet —
this is the menu, not a commitment to any one of them.

## Where the two cloned repos fit

Neither repo does world-mapping or splatting itself — both are adjacent, already-real pieces
this stack would sit between:

- **Witness-firmware** (`cryptonomicsed-byte/Witness-firmware`) — the attestation layer.
  LoRa physics-proof attestation (payload hash + RSSI + timestamp), hash-linked chain, mesh
  gossip consensus. Its role here: proving a capture-contributing device was physically where
  and when it claims to be, before its delta gets trusted into the twin. This is presence/time
  attestation, not image capture.
- **ScarabSwarm** (`cryptonomicsed-byte/ScarabSwarm`) — the simulation/training engine for
  drone-type embodied agents (Julia, RigidBodyDynamics 6-DOF flight physics, SHA256 trajectory
  proofs). Its role here: once the twin exists, this is what a drone-sector agent trains in
  against it before real embodiment — the PoSim half of the sim→real loop for that specific
  embodiment class, not the twin-building pipeline itself.

## Revenue loop

blob capture (device) → attestation (Witness-firmware) → Walrus storage (mineable PoSim job) →
twin data sale over time → proceeds route back through Èṣù's 3.69% router → funds the 24-sector
shrine-funding loop (`docs/OSOVM_CODEX.md` §13-17) that pays capture-contributing devices and
underwrites new embodiment.

## Open, not decided

- Which DePIN network(s) to actually integrate first — Hivemapper looks like the closest
  existing fit for imagery specifically, but nothing is chosen.
- Splatting pipeline ownership (Rust/Julia native vs. wrapping an existing OSS splatting
  library) — not evaluated yet.
- How attestation failure (a device's claimed location/time doesn't check out) degrades or
  rejects a delta contribution — not specified.
- Blocked on task #6 (cross-machine VeilSim determinism) regardless of the above.
