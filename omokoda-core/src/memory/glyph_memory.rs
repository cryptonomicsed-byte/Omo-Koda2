//! Glyph-memory projection — Ọmọ Kọ́dà's leg of the ecosystem **GlyphIndex**
//! contract (see `OSOVM/GLYPHINDEX_SPEC.md`, which assigns this repo the
//! "SOMA/REM integration, birth registration" role).
//!
//! This projects the agent's sovereign Odù memory ([`OduDirectory`]) into a
//! [`larql_glyph::GlyphGraph`] — the *same* content-addressed metadata graph
//! that mnemopi (TypeScript), larql (Rust), and zerolang produce, proven
//! byte-compatible against the canonical Python reference via frozen vectors.
//!
//! ## Why this is safe (additive, never a downgrade)
//! - It is **read-only**: `project` borrows the directory and builds a *new*
//!   graph. Nothing in the existing memory path changes.
//! - Only **metadata** crosses the wire: content-addressed glyph, Odù linkage,
//!   the tags the agent already chose, edges, and an `omokoda://` locator.
//!   Plaintext stays in the vault; a `WALK`/`DESCRIBE` hit is expanded back to
//!   text locally via the locator, exactly like mnemopi's `mnemopi://` scheme.
//! - It reuses the `larql-glyph` crate this workspace **already** depends on
//!   (`memdir.rs` already content-addresses with `larql_glyph::content_hash`),
//!   so there is no new dependency and no new wire format to maintain.
//!
//! ## Not yet wired (needs a `larql-glyph` bump, tracked separately)
//! `GlyphGraph::merge` (agent-to-agent exchange) and `merkle_root` / `gix1_audit`
//! (Sui anchoring + keyless audit) landed in later `larql-glyph` commits than the
//! pinned rev. Adopting them is a clean follow-up: bump the pin, then add a
//! `glyph_merge` path here and anchor through the existing `walrus.rs`.

use larql_glyph::{GlyphGraph, GlyphNode};

// Re-exported anchoring surface (pinned larql-glyph rev 149322e). Ọmọ Kọ́dà does
// not seal blobs itself (that is the crypto leg — BIPON39/Zangbeto/Vantage), but
// once a sealing path exists (via `walrus.rs`), these compute the keyless audit
// and the Sui-anchorable Merkle root from `(canonical_id, blob_sha256)` receipts.
pub use larql_glyph::{gix1_audit, merkle_root, GIX1_EMPTY_ROOT};

use crate::memory::memdir::{OduDirectory, OduEntry};

/// Locator that expands a graph node back to its plaintext entry in *this*
/// agent's vault — the Ọmọ Kọ́dà analog of mnemopi's `mnemopi://<bank>/<id>`.
/// Only the locator (never the plaintext) is placed on the node.
fn locator(owner: &str, entry: &OduEntry) -> String {
    format!("omokoda://{owner}/{}", entry.id)
}

/// Project an [`OduDirectory`] into a [`GlyphGraph`].
///
/// Deterministic: entries are visited in `id` order so the serialized snapshot
/// is stable across runs (and thus diffable / fixture-comparable). Edges:
///   - `"follows"` — episodic chain: within each vault `path` (a conversation /
///     session cluster), consecutive entries by `created_at` are linked, the
///     same shape as mnemopi's per-session follows-chains.
///   - `"shared-odu"` — semantic: materialized by `infer_shared_odu` over the
///     Digital Calabash base-Odù, Ọmọ Kọ́dà's native memory linkage.
pub fn project(dir: &OduDirectory, owner: &str) -> GlyphGraph {
    let mut graph = GlyphGraph::new();

    let mut entries: Vec<&OduEntry> = dir.entries.values().collect();
    entries.sort_by(|a, b| a.id.cmp(&b.id));

    // Insert one node per memory entry. `canonical_id` is the SHA-256 of the
    // content (GIX-FOLD-v1), so two entries with identical text fold to one
    // node — that is the content-addressing guarantee, not data loss.
    for entry in &entries {
        let mut node = GlyphNode::from_chunk(&entry.content, entry.created_at as f64);
        for tag in &entry.tags {
            node.tags.insert(tag.clone());
        }
        // Reuse the node's expansion-locator slot for our vault URI.
        node.walrus_blob_id = Some(locator(owner, entry));
        // Ignore BadCanonicalId: from_chunk always yields a 64-hex id.
        let _ = graph.insert(node);
    }

    // Episodic "follows" edges within each path cluster.
    let mut by_path: std::collections::BTreeMap<&str, Vec<&OduEntry>> =
        std::collections::BTreeMap::new();
    for entry in &entries {
        by_path.entry(entry.path.as_str()).or_default().push(entry);
    }
    for cluster in by_path.values() {
        let mut chain: Vec<&OduEntry> = cluster.clone();
        // Stable order: by creation time, then id to break ties.
        chain.sort_by(|a, b| a.created_at.cmp(&b.created_at).then(a.id.cmp(&b.id)));
        for pair in chain.windows(2) {
            let from = hex::encode(larql_glyph::content_hash(&pair[0].content));
            let to = hex::encode(larql_glyph::content_hash(&pair[1].content));
            if from != to {
                // Both nodes are present (inserted above); a self-loop from
                // identical folded content is skipped.
                let _ = graph.link(&from, &to, "follows");
            }
        }
    }

    // Semantic shared-Odù edges (Ọmọ Kọ́dà's native calabash linkage).
    graph.infer_shared_odu();

    graph
}

/// Serialized snapshot of the projected graph — the interop payload other eco
/// legs (mnemopi / larql / zerolang / Axiom) consume.
pub fn snapshot_json(dir: &OduDirectory, owner: &str) -> serde_json::Value {
    project(dir, owner).to_json()
}

// Agent-to-agent memory-graph merge is `larql_glyph::GlyphGraph::merge` (pinned
// rev 149322e): tags union, earliest `ts` wins, locators never dropped, edges
// unioned, idempotent. Call it directly on the projected graph — no Ọmọ Kọ́dà
// reimplementation is maintained now that upstream ships the same contract.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::memdir::{OduDirectory, OduEntry};

    /// The pinned `larql-glyph` produces the spec's frozen canonical glyphs —
    /// this is Ọmọ Kọ́dà proving it speaks GIX-FOLD-v1 byte-for-byte.
    #[test]
    fn fold_matches_frozen_spec_vectors() {
        for (text, codepoint, base, composed) in [
            ("Àṣẹ", 21841u32, 227u8, 58152u16),
            ("hello", 23636, 44, 11506),
            ("GlyphIndex", 13726, 68, 17595),
            ("Ọ̀rúnmìlà", 17963, 204, 52390),
        ] {
            let digest = larql_glyph::content_hash(text);
            assert_eq!(larql_glyph::glyph_fold(&digest) as u32, codepoint);
            assert_eq!(larql_glyph::odu_link(&digest), (base, composed));
        }
    }

    fn dir_with(entries: &[(&str, &str, &str)]) -> OduDirectory {
        let mut dir = OduDirectory::new();
        for (i, (content, path, tag)) in entries.iter().enumerate() {
            let mut e = OduEntry::new(format!("e{i}"), *content, *path);
            e.created_at = 1000 + i as u64; // deterministic ordering
            e.tags = vec![tag.to_string()];
            dir.insert(e);
        }
        dir
    }

    #[test]
    fn projection_is_deterministic_and_metadata_only() {
        let dir = dir_with(&[
            ("first thought", "chat/session-a", "topic:alpha"),
            ("second thought", "chat/session-a", "topic:alpha"),
            ("unrelated", "chat/session-b", "topic:beta"),
        ]);
        let g1 = snapshot_json(&dir, "agent-x");
        let g2 = snapshot_json(&dir, "agent-x");
        assert_eq!(g1, g2, "projection must be deterministic");

        // No plaintext leaks into the serialized graph.
        let s = g1.to_string();
        assert!(!s.contains("first thought"));
        assert!(!s.contains("second thought"));
        // Locator and tags are present (safe metadata).
        assert!(s.contains("omokoda://agent-x/"));
        assert!(s.contains("topic:alpha"));
    }

    /// Cross-language golden fixture: the *exact* byte-identical snapshot the
    /// other eco legs (mnemopi/TS, larql/Rust, zerolang) round-trip against
    /// (committed from `cryptonomicsed-byte/larql` @149322e, git blob c35b08c6).
    /// This proves an Ọmọ Kọ́dà runtime reads a TS-generated graph, re-serializes
    /// it to the same canonical form, and reproduces shared WALK/DESCRIBE.
    #[test]
    fn golden_fixture_roundtrips_and_walks() {
        let raw = include_str!("../../tests/fixtures/glyph-graph-snapshot.json");
        let graph: GlyphGraph = serde_json::from_str(raw).expect("TS fixture deserializes in Rust");
        assert_eq!(graph.len(), 5);

        // Re-serialization is a stable canonical form (deterministic wire).
        let once = serde_json::to_string(&graph).unwrap();
        let twice =
            serde_json::to_string(&serde_json::from_str::<GlyphGraph>(&once).unwrap()).unwrap();
        assert_eq!(once, twice, "serialization must be canonical/stable");

        // The fixture's nodes are exactly GIX-FOLD-v1 of their source text.
        let hello = hex::encode(larql_glyph::content_hash("hello"));
        assert_eq!(
            hello,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
        assert!(graph.describe(&hello).is_ok());

        // Shared WALK behavior: hello --follows--> GlyphIndex.
        let gidx = hex::encode(larql_glyph::content_hash("GlyphIndex"));
        let reached = graph.walk(&hello, 1).unwrap();
        assert!(
            reached.iter().any(|n| n.canonical_id == gidx),
            "WALK must reproduce the cross-language follows edge"
        );
    }

    #[test]
    fn merge_unions_tags_and_is_idempotent() {
        // Two agents each hold the same memory ("shared fact") with different
        // tags, plus one private memory each.
        let dir_a = dir_with(&[("shared fact", "p", "from:a"), ("only a", "p", "priv:a")]);
        let dir_b = dir_with(&[("shared fact", "q", "from:b"), ("only b", "q", "priv:b")]);
        let mut a = project(&dir_a, "a");

        let before = a.len();
        // upstream merge consumes `other`; project a fresh b (deterministic).
        a.merge(project(&dir_b, "b"));
        let after = a.len();
        // 2 (a) + 2 (b) - 1 shared = 3 distinct content nodes.
        assert_eq!(after, 3, "shared content-addressed node must not duplicate");
        assert!(after > before);

        // The shared node carries tags from BOTH agents (union).
        let shared_id = hex::encode(larql_glyph::content_hash("shared fact"));
        let desc = a.describe(&shared_id).unwrap();
        assert!(desc.node.tags.contains("from:a"));
        assert!(desc.node.tags.contains("from:b"));

        // Idempotent: merging b again changes nothing.
        let snap = a.to_json();
        a.merge(project(&dir_b, "b"));
        assert_eq!(a.to_json(), snap, "merge must be idempotent");
    }

    #[test]
    fn follows_chain_links_same_path_entries() {
        let dir = dir_with(&[
            ("turn one", "chat/s", "t"),
            ("turn two", "chat/s", "t"),
        ]);
        let graph = project(&dir, "owner");
        let from = hex::encode(larql_glyph::content_hash("turn one"));
        // WALK depth-1 from the first entry reaches the second via "follows".
        let reached = graph.walk(&from, 1).unwrap();
        assert!(reached
            .iter()
            .any(|n| n.canonical_id == hex::encode(larql_glyph::content_hash("turn two"))));
    }
}
