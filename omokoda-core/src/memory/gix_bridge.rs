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

    // Semantic "recalls" edges — entries sharing ≥1 common non-trivial tag.
    // Two memories that share a tag co-activate in recall; weight 2 reflects
    // stronger semantic coupling than the temporal "follows" chain.
    {
        let mut tag_to_ids: std::collections::BTreeMap<String, Vec<String>> =
            std::collections::BTreeMap::new();
        for entry in &entries {
            let hex = hex::encode(gix_types::content_hash(&entry.content));
            for tag in &entry.tags {
                if !tag.is_empty() {
                    tag_to_ids.entry(tag.clone()).or_default().push(hex.clone());
                }
            }
        }
        for ids in tag_to_ids.values() {
            for pair in ids.windows(2) {
                if pair[0] != pair[1] {
                    graph.add_edge(GixEdge {
                        from:     pair[0].clone(),
                        to:       pair[1].clone(),
                        relation: "recalls".to_string(),
                        weight:   2,
                    });
                }
            }
        }
    }

    // Semantic "derives" edges — a sub-path entry derives from the most-recent
    // entry in its parent path cluster, reflecting hierarchical memory synthesis.
    {
        for entry in &entries {
            if let Some(parent_path) = entry.path.rsplit_once('/').map(|(p, _)| p) {
                if let Some(cluster) = by_path.get(parent_path) {
                    // Pick the most recent parent entry (latest created_at).
                    if let Some(parent) = cluster.iter().max_by_key(|e| e.created_at) {
                        let from = hex::encode(gix_types::content_hash(&parent.content));
                        let to   = hex::encode(gix_types::content_hash(&entry.content));
                        if from != to {
                            graph.add_edge(GixEdge {
                                from,
                                to,
                                relation: "derives".to_string(),
                                weight:   1,
                            });
                        }
                    }
                }
            }
        }
    }

    // Semantic "contradicts" edges — entries tagged with a failure/error marker
    // point back to their immediate predecessor in the same path cluster.
    // Negative weight (-1) signals that this edge inverts the preceding assertion.
    {
        const CONTRADICTION_TAGS: &[&str] = &["error", "fail", "failure", "contradiction", "rejected"];
        for cluster in by_path.values() {
            let mut chain: Vec<&crate::memory::memdir::OduEntry> = cluster.clone();
            chain.sort_by(|a, b| a.created_at.cmp(&b.created_at).then(a.id.cmp(&b.id)));
            for (pos, entry) in chain.iter().enumerate() {
                let has_contradiction = entry.tags.iter()
                    .any(|t| CONTRADICTION_TAGS.contains(&t.as_str()));
                if !has_contradiction || pos == 0 { continue; }
                let pred = chain[pos - 1];
                let from = hex::encode(gix_types::content_hash(&pred.content));
                let to   = hex::encode(gix_types::content_hash(&entry.content));
                if from != to {
                    graph.add_edge(GixEdge {
                        from,
                        to,
                        relation: "contradicts".to_string(),
                        weight:   -1,
                    });
                }
            }
        }
    }

    graph
}

// ── GIX query API ────────────────────────────────────────────────────────────

/// Query result from a GIX SELECT / DESCRIBE / WALK / INFER operation.
#[derive(Debug, Clone)]
pub struct GixSelectResult {
    /// Hex SHA-256 canonical id of the entry.
    pub id: String,
    /// GIX kind discriminant.
    pub kind: GixKind,
    /// Unix timestamp (float seconds) when the entry was recorded.
    pub timestamp: f64,
}

impl GixSelectResult {
    fn from_entry(e: &Gix1Entry) -> Self {
        Self {
            id: e.canonical_id.clone(),
            kind: e.kind.clone(),
            timestamp: e.ts,
        }
    }
}

/// DESCRIBE: return all metadata for a single entry by canonical id.
/// Returns `None` if the entry is not present in the index.
pub fn query_describe(index: &Gix1Index, id: &str) -> Option<GixSelectResult> {
    index.entries().iter().find(|e| e.canonical_id == id).map(GixSelectResult::from_entry)
}

/// SELECT: filter entries by optional kind and optional time range \[since_ts, until_ts\].
/// Pass `None` for either bound to leave it open.
pub fn query_select(
    index: &Gix1Index,
    kind: Option<&GixKind>,
    since_ts: Option<f64>,
    until_ts: Option<f64>,
) -> Vec<GixSelectResult> {
    index
        .entries()
        .iter()
        .filter(|e| kind.map_or(true, |k| &e.kind == k))
        .filter(|e| since_ts.map_or(true, |t| e.ts >= t))
        .filter(|e| until_ts.map_or(true, |t| e.ts <= t))
        .map(GixSelectResult::from_entry)
        .collect()
}

/// WALK: traverse from a starting canonical id, following entries within
/// `max_depth` timestamp steps (each step = entries within ±1.0 ts of the
/// previous frontier).  Returns the traversal path as a list of
/// `GixSelectResult`.
///
/// Because `Gix1Index` has no explicit edge structure, proximity is defined
/// purely by timestamp adjacency (±1.0 seconds per hop).
pub fn query_walk(index: &Gix1Index, from_id: &str, max_depth: usize) -> Vec<GixSelectResult> {
    // Find the starting entry.
    let start = match index.entries().iter().find(|e| e.canonical_id == from_id) {
        Some(e) => e,
        None => return vec![],
    };

    let mut visited: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut result: Vec<GixSelectResult> = Vec::new();
    let mut frontier_ts: Vec<f64> = vec![start.ts];

    visited.insert(start.canonical_id.clone());
    result.push(GixSelectResult::from_entry(start));

    for _ in 0..max_depth {
        let mut next_frontier_ts: Vec<f64> = Vec::new();
        for &center_ts in &frontier_ts {
            for entry in index.entries().iter() {
                if visited.contains(&entry.canonical_id) {
                    continue;
                }
                if (entry.ts - center_ts).abs() <= 1.0 {
                    visited.insert(entry.canonical_id.clone());
                    next_frontier_ts.push(entry.ts);
                    result.push(GixSelectResult::from_entry(entry));
                }
            }
        }
        if next_frontier_ts.is_empty() {
            break;
        }
        frontier_ts = next_frontier_ts;
    }

    result
}

/// INFER: given a partial id prefix, return up to `limit` entries whose
/// canonical id starts with `id_prefix` (prefix match on the hex string).
/// Used for semantic / auto-completion queries.
pub fn query_infer(index: &Gix1Index, id_prefix: &str, limit: usize) -> Vec<GixSelectResult> {
    index
        .entries()
        .iter()
        .filter(|e| e.canonical_id.starts_with(id_prefix))
        .take(limit)
        .map(GixSelectResult::from_entry)
        .collect()
}

/// Generate a simple Merkle inclusion proof for an entry.
///
/// Returns `(leaf_hash, root_hash, siblings_hex)` where `siblings_hex` is the
/// ordered list of sibling hashes needed to reconstruct the root from the leaf.
///
/// Returns `Err` if the entry is not found in the index.
///
/// Implementation note: this replicates the same pairwise SHA-256 reduction
/// used by `gix1_merkle_root` in `gix_types`.  We build the proof bottom-up
/// so callers can independently verify inclusion without holding the full index.
pub fn merkle_proof(
    index: &Gix1Index,
    id: &str,
) -> Result<(String, String, Vec<String>), String> {
    use sha2::{Digest, Sha256};

    if index.is_empty() {
        return Err(format!("index is empty; entry '{id}' not found"));
    }

    // Sort canonical_ids the same way gix1_merkle_root does.
    let mut sorted_ids: Vec<&str> = index
        .entries()
        .iter()
        .map(|e| e.canonical_id.as_str())
        .collect();
    sorted_ids.sort_unstable();

    let leaf_pos = sorted_ids
        .iter()
        .position(|&s| s == id)
        .ok_or_else(|| format!("entry '{id}' not found in index"))?;

    // Build the leaf layer: hash each canonical_id.
    let mut layer: Vec<[u8; 32]> = sorted_ids
        .iter()
        .map(|s| {
            let mut h = Sha256::new();
            h.update(s.as_bytes());
            h.finalize().into()
        })
        .collect();

    let leaf_hash = hex::encode(layer[leaf_pos]);
    let mut siblings: Vec<String> = Vec::new();
    let mut pos = leaf_pos;

    while layer.len() > 1 {
        // Find this node's sibling at the current level.
        let sibling_pos = if pos % 2 == 0 {
            // Left node: sibling is to the right (or self if odd layer).
            if pos + 1 < layer.len() { pos + 1 } else { pos }
        } else {
            // Right node: sibling is to the left.
            pos - 1
        };
        siblings.push(hex::encode(layer[sibling_pos]));

        // Reduce layer.
        let mut next: Vec<[u8; 32]> = Vec::with_capacity((layer.len() + 1) / 2);
        let mut i = 0;
        while i < layer.len() {
            let left  = layer[i];
            let right = if i + 1 < layer.len() { layer[i + 1] } else { left };
            let mut h = Sha256::new();
            h.update(&left);
            h.update(&right);
            next.push(h.finalize().into());
            i += 2;
        }
        pos /= 2;
        layer = next;
    }

    let root_hash = hex::encode(layer[0]);
    Ok((leaf_hash, root_hash, siblings))
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

#[cfg(test)]
mod gix_bridge_tests {
    use super::*;
    use crate::memory::memdir::{OduDirectory, OduEntry};

    fn make_dir(entries: Vec<OduEntry>) -> OduDirectory {
        let mut dir = OduDirectory::new();
        for e in entries {
            dir.insert(e);
        }
        dir
    }

    fn entry(id: &str, content: &str, path: &str, ts: u64, tags: &[&str]) -> OduEntry {
        OduEntry {
            id: id.into(),
            content: content.into(),
            importance: 0.5,
            created_at: ts,
            last_accessed: ts,
            tags: tags.iter().map(|t| t.to_string()).collect(),
            path: path.into(),
        }
    }

    #[test]
    fn follows_edges_appear_within_path_cluster() {
        let dir = make_dir(vec![
            entry("e1", "alpha", "memory/core", 100, &[]),
            entry("e2", "beta",  "memory/core", 200, &[]),
            entry("e3", "gamma", "memory/other", 300, &[]),
        ]);
        let g = project_gix(&dir);
        let edges: Vec<_> = g.edges().iter()
            .filter(|e| e.relation == "follows").collect();
        // e1→e2 follow edge; e3 is in a different cluster — no cross-cluster follows
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].weight, 1);
    }

    #[test]
    fn recalls_edges_link_entries_with_shared_tag() {
        let dir = make_dir(vec![
            entry("e1", "alpha content", "memory/a", 100, &["session", "core"]),
            entry("e2", "beta content",  "memory/b", 200, &["session"]),
            entry("e3", "gamma content", "memory/c", 300, &["unrelated"]),
        ]);
        let g = project_gix(&dir);
        let recalls: Vec<_> = g.edges().iter()
            .filter(|e| e.relation == "recalls").collect();
        // e1 and e2 both have tag "session" → 1 recalls edge
        assert_eq!(recalls.len(), 1);
        assert_eq!(recalls[0].weight, 2);
    }

    #[test]
    fn derives_edges_link_child_path_to_parent() {
        let dir = make_dir(vec![
            entry("e1", "parent content", "memory",      100, &[]),
            entry("e2", "child content",  "memory/core", 200, &[]),
        ]);
        let g = project_gix(&dir);
        let derives: Vec<_> = g.edges().iter()
            .filter(|e| e.relation == "derives").collect();
        // e2 is at "memory/core", parent is "memory" which contains e1
        assert_eq!(derives.len(), 1);
        assert_eq!(derives[0].weight, 1);
    }

    #[test]
    fn contradicts_edges_have_negative_weight() {
        let dir = make_dir(vec![
            entry("e1", "assertion", "memory/core", 100, &[]),
            entry("e2", "error log", "memory/core", 200, &["error"]),
        ]);
        let g = project_gix(&dir);
        let contradicts: Vec<_> = g.edges().iter()
            .filter(|e| e.relation == "contradicts").collect();
        assert_eq!(contradicts.len(), 1);
        assert_eq!(contradicts[0].weight, -1);
    }

    #[test]
    fn no_self_edges_in_any_relation() {
        let dir = make_dir(vec![
            entry("e1", "unique alpha",   "memory/core", 100, &["tag"]),
            entry("e2", "unique alpha",   "memory/core", 200, &["tag", "error"]),
        ]);
        let g = project_gix(&dir);
        for edge in g.edges() {
            assert_ne!(edge.from, edge.to, "self-edge found: {:?}", edge);
        }
    }
}
