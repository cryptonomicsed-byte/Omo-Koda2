# GlyphIndex Integration — Ọmọ Kọ́dà's leg

Ọmọ Kọ́dà participates in the ecosystem-wide **GlyphIndex** sovereign-memory
contract (canonical spec: `OSOVM/GLYPHINDEX_SPEC.md`; canonical reference:
`Vantage/backend/glyph_index.py`). The spec assigns this repo the
**"SOMA/REM integration, birth registration"** role — i.e. projecting the
agent's living Odù memory into the shared, content-addressed graph that
mnemopi (TypeScript), larql (Rust), and zerolang also speak.

## What was added (this phase)

Purely **additive and read-only** — no existing memory behaviour changed, so
nothing is broken or downgraded.

- **`omokoda-core/src/memory/glyph_memory.rs`** — `project(&OduDirectory, owner)
  -> larql_glyph::GlyphGraph`, plus `snapshot_json`. Each memory entry becomes a
  content-addressed `GlyphNode` (GIX-FOLD-v1 glyph + Odù linkage), tagged with
  the entry's own tags and an `omokoda://<owner>/<id>` expansion locator.
  Edges: `"follows"` (episodic chains within a vault path) and `"shared-odu"`
  (semantic, via the Digital Calabash base-Odù). Plaintext never enters the graph.
- **`AgentCore::glyph_memory()`** (`interpreter.rs`) — projects the live
  `snapshot.odu_dir` for the current agent.
- **`GET /v1/vault/glyph`** (`server.rs`) — serves the projection as JSON.
  Optional `?describe=<canonical_id>` and `?walk=<canonical_id>&depth=<n>`;
  `x-agent-id` header selects a guest agent. This is the interop surface Axiom
  and other-eco agents pull.
- **`POST /v1/vault/glyph/merge`** (`server.rs`) — agent-to-agent exchange: body
  is another agent's snapshot (as from `GET`); returns it merged into this
  agent's projection (`memory::glyph_memory::merge`). The caller's sealed memory
  is untouched — only the returned graph reflects the union.

## Why this is byte-compatible

Ọmọ Kọ́dà already depends on the **`larql-glyph`** crate and already uses its
`content_hash` in `memdir.rs`. The projection reuses that crate's
`GlyphNode` / `GlyphGraph` directly, so the serialized snapshot is identical to
larql's own — the frozen cross-language vectors (`Àṣẹ`, `hello`, `GlyphIndex`,
`Ọ̀rúnmìlà`, …) are asserted in `glyph_memory.rs`'s tests and pass. UTF-8 byte
ordering via `BTreeMap`/`BTreeSet` matches the canonical reference.

## Dependency pin — bumped to the merge/anchor rev (2026-07-21)

`omokoda-core/Cargo.toml` pins `larql-glyph` at
`149322efa11a2592b9cbebfbfa91c98d7b2d50a7` (branch
`claude/agent-native-architecture-nd2m8c` of `cryptonomicsed-byte/larql`), which
adds `GlyphGraph::merge`, `gix1_audit`, `merkle_root`, and `GIX1_EMPTY_ROOT` on
top of the frozen GIX-FOLD/graph baseline. The frozen wire functions
(`content_hash` / `glyph_fold` / `odu_link`) are unchanged — the conformance
test in `glyph_memory.rs` re-asserts the spec vectors after the bump.

## Agent-to-agent `merge`

`GlyphGraph::merge(other)` (upstream) is used directly — **tags union, earliest
`ts` wins, locators never dropped, edges unioned, idempotent**. Ọmọ Kọ́dà keeps
no parallel reimplementation. `POST /v1/vault/glyph/merge` calls it.

## Anchoring / audit — available, but sealing belongs to the crypto leg

`gix1_audit(blob)` and `merkle_root(&[(canonical_id, blob_sha256)])` (+
`GIX1_EMPTY_ROOT`) are re-exported from `memory::glyph_memory`. The Merkle root
is over *sealed-blob* hashes, so producing the actual anchor requires GIX1
sealing — the spec assigns that crypto to **BIPON39 / Zangbeto / Cloakseed /
Vantage**, not this repo. Ọmọ Kọ́dà supplies the ordered `canonical_id` set and
can *audit/verify* blobs and recompute a root from receipts, but does not seal.
Its own sealed private memory (ChaCha20Poly1305 + `odu_keys` HKDF chain) is
unrelated and unchanged.

GIX **crypto** (GIX-KDF-v1 keyrings, GIX1 AES-256-GCM envelopes) is *not* this
repo's job per the spec — it lives in BIPON39 / Zangbeto / Cloakseed. Ọmọ Kọ́dà's
own sealed private memory (ChaCha20Poly1305 + the `odu_keys` HKDF chain) stays
as-is; the GlyphIndex leg is the *queryable projection* on top, not a
replacement for it.
