//! GIX bridge — wires gix-core (Glyph Index Runtime) into Ọmọ Kọ́dà's memory
//! layer for Merkle-auditable memory snapshots and keyless Sui anchoring.
//!
//! ## What this adds vs glyph_memory.rs
//! `glyph_memory.rs` uses `larql_glyph` (pinned rev 149322e) which does not yet
//! ship `gix1_audit` / `gix1_merkle_root`. This module pulls those from the local
//! `gix-core` path crate (~/GIX) which is the canonical sovereign implementation.
//!
//! ## What stays in glyph_memory.rs
//! `project()` / `snapshot_json()` / `filter_snapshot()` / `anchor_entries()` —
//! all larql-glyph graph operations stay there. This file only adds the
//! audit/Merkle/Gix1Index layer on top of those projections.

pub use gix_core::{
    Gix1Index,
    GlyphGraph as GixGraph,
    gix1_audit, gix1_merkle_root, GIX1_EMPTY_ROOT,
    GlyphNode as GixNode, GlyphEdge as GixEdge, GixKind, Gix1Entry,
};

use crate::memory::memdir::OduDirectory;

/// Compute a GIX1 Merkle root over the agent's current memory directory.
/// The root is deterministic: same directory contents always yields the same root,
/// regardless of insertion order (entries are sorted by canonical_id before hashing).
pub fn memory_merkle_root(dir: &OduDirectory) -> String {
    let canonical_ids = dir_canonical_ids(dir);
    let id_refs: Vec<&str> = canonical_ids.iter().map(|s| s.as_str()).collect();
    gix1_merkle_root(&id_refs)
}

/// Audit an existing stored root against the current directory state.
/// Returns `Ok(root)` if consistent, `Err(message)` on mismatch.
pub fn audit_memory_root(dir: &OduDirectory, stored_root: &str) -> Result<String, String> {
    let canonical_ids = dir_canonical_ids(dir);
    let id_refs: Vec<&str> = canonical_ids.iter().map(|s| s.as_str()).collect();
    gix1_audit(stored_root, &id_refs)
}

/// Build a `Gix1Index` from the agent's memory directory.
/// Each Odù entry becomes a `GixKind::Memory` entry in the index.
pub fn build_gix1_index(dir: &OduDirectory) -> Gix1Index {
    let mut index = Gix1Index::new();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();

    // Stable insertion order: sort by entry id so the index is deterministic.
    let mut ids: Vec<&str> = dir.entries.keys().map(|k| k.as_str()).collect();
    ids.sort_unstable();

    for id in ids {
        if let Some(entry) = dir.entries.get(id) {
            index.add_receipt(&entry.id, GixKind::Memory, entry.created_at as f64);
        }
    }

    // Verify the freshly-built index is self-consistent before returning.
    debug_assert!(
        index.audit().is_ok(),
        "gix_bridge: freshly-built index failed self-audit"
    );

    let _ = now; // suppress unused warning — available for caller timestamping
    index
}

/// Add a receipt hash (any kind) to an existing Gix1Index and return the new root.
pub fn record_receipt(index: &mut Gix1Index, receipt_id: &str, kind: GixKind, ts: f64) -> String {
    index.add_receipt(receipt_id, kind, ts);
    index.root().to_string()
}

/// Build a `GixGraph` from the agent's memory directory using `gix-types` nodes.
///
/// This is the `gix-core` flavor of the projection in `glyph_memory::project()`.
/// Both graphs represent the same memory; the difference is the type system:
/// - `glyph_memory::project()` → `larql_glyph::GlyphGraph` (LQL queries)
/// - this function → `gix_core::GlyphGraph` (DESCRIBE/SELECT/WALK/INFER + audit)
pub fn project_gix(dir: &OduDirectory) -> GixGraph {
    let mut graph = GixGraph::new();

    let mut entries: Vec<&crate::memory::memdir::OduEntry> = dir.entries.values().collect();
    entries.sort_by(|a, b| a.id.cmp(&b.id));

    for entry in &entries {
        let mut node = GixNode::from_chunk(&entry.content, entry.created_at as f64);
        for tag in &entry.tags {
            node.tags.insert(tag.clone());
        }
        graph.add_node(node);
    }

    // Episodic "follows" edges within each path cluster (same as glyph_memory::project).
    let mut by_path: std::collections::BTreeMap<&str, Vec<&crate::memory::memdir::OduEntry>> =
        std::collections::BTreeMap::new();
    for entry in &entries {
        by_path.entry(entry.path.as_str()).or_default().push(entry);
    }
    for cluster in by_path.values() {
        let mut chain: Vec<&crate::memory::memdir::OduEntry> = cluster.clone();
        chain.sort_by(|a, b| a.created_at.cmp(&b.created_at).then(a.id.cmp(&b.id)));
        for pair in chain.windows(2) {
            let from_digest = gix_types::content_hash(&pair[0].content);
            let to_digest   = gix_types::content_hash(&pair[1].content);
            let from = hex::encode(from_digest);
            let to   = hex::encode(to_digest);
            if from != to {
                graph.add_edge(GixEdge { from, to, relation: "follows".to_string(), weight: 1 });
            }
        }
    }

    graph
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn dir_canonical_ids(dir: &OduDirectory) -> Vec<String> {
    let mut ids: Vec<String> = dir.entries.values()
        .map(|e| hex::encode(gix_types::content_hash(&e.content)))
        .collect();
    ids.sort_unstable();
    ids.dedup();
    ids
}
