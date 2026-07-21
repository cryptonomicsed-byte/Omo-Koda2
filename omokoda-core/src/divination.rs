//! Real LARQL-style divination over an agent's own memory.
//!
//! Wires `larql-glyph` -- a small, self-contained graph library, not the
//! full model-serving `larql-server` (which needs multi-GB `.vindex` model
//! data that doesn't exist anywhere in this stack, and wasn't deployed for
//! that reason; see the memory `larql-server` task). No plaintext is
//! retained in the graph: `GlyphNode::from_chunk` stores only a
//! content-hash-derived glyph char and Odù link, never the message text
//! itself, mirroring the "sealed memory" design intent this ecosystem's
//! docs describe elsewhere.
//!
//! `infer_shared_odu()` is the real divinatory signal: it finds message
//! pairs whose Odù lineage (derived from the content hash) shares a base
//! byte -- a genuine, if simple, recurring-pattern detector over an
//! agent's own conversation history, not a fabricated "AI intuition."

use larql_glyph::GlyphGraph;

use crate::session::{ContentBlock, ConversationMessage};

fn message_text(message: &ConversationMessage) -> String {
    message
        .blocks
        .iter()
        .filter_map(|b| match b {
            ContentBlock::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Build a glyph graph from an agent's own conversation history: one node
/// per message (skipping empty/tool-only messages), linked in sequence,
/// then run INFER to surface any shared-Odù recurrence.
pub fn build_memory_graph(messages: &[ConversationMessage]) -> GlyphGraph {
    let mut graph = GlyphGraph::new();
    let mut previous_id: Option<String> = None;

    for message in messages {
        let text = message_text(message);
        if text.trim().is_empty() {
            continue;
        }
        let node = larql_glyph::GlyphNode::from_chunk(&text, message.timestamp as f64);
        let id = node.canonical_id.clone();
        // insert() only fails on a malformed canonical_id, which
        // from_chunk always produces validly (64 hex chars) -- an error
        // here would mean from_chunk's own hex::encode output changed
        // shape, not a real runtime condition to recover from.
        if graph.insert(node).is_ok() {
            if let Some(prev) = &previous_id {
                let _ = graph.link(prev, &id, "follows");
            }
            previous_id = Some(id);
        }
    }

    graph.infer_shared_odu();
    graph
}

/// The most frequently occurring glyph across an agent's own memory
/// graph, folded to a single byte for the on-chain glyph-signal field
/// (see onchain.rs::update_onchain_glyph_signal). Each glyph char comes
/// from a real, broad Unicode fold range (larql_glyph::glyph_fold), not
/// guaranteed ASCII, so this uses `codepoint % 256` rather than a lossy
/// truncation that would silently drop non-ASCII glyphs to garbage.
/// `None` for an empty graph.
pub fn dominant_glyph_byte(graph: &GlyphGraph) -> Option<u8> {
    let nodes = graph.to_json().get("nodes")?.as_object()?.clone();
    // BTreeMap, not HashMap: HashMap's iteration order is randomized
    // per-process, so a tie in counts (e.g. two distinct glyphs each
    // appearing once) would pick a different "dominant" byte on every
    // run -- confirmed live as a real test flake before this fix.
    // BTreeMap's ascending byte-value iteration makes ties break the
    // same way every time.
    let mut counts: std::collections::BTreeMap<u8, usize> = std::collections::BTreeMap::new();
    for node in nodes.values() {
        let glyph_str = node.get("glyph")?.as_str()?;
        let ch = glyph_str.chars().next()?;
        let byte = (ch as u32 % 256) as u8;
        *counts.entry(byte).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(byte, _)| byte)
}

/// A real, minimal divinatory signal from an agent's own memory graph:
/// how many shared-Odù recurrences exist. Returns `None` for an empty or
/// pattern-free graph rather than a zero-value string, so callers can
/// skip the prompt line entirely instead of stating "no pattern found."
pub fn recurrence_signal(graph: &GlyphGraph) -> Option<usize> {
    if graph.is_empty() {
        return None;
    }
    let shared = graph
        .to_json()
        .get("edges")
        .and_then(|e| e.as_array())
        .map(|edges| {
            edges
                .iter()
                .filter(|e| e.get("relation").and_then(|r| r.as_str()) == Some("shared-odu"))
                .count()
        })
        .unwrap_or(0);
    (shared > 0).then_some(shared)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::MessageRole;

    fn text_message(text: &str, ts: u64) -> ConversationMessage {
        ConversationMessage {
            role: MessageRole::User,
            blocks: vec![ContentBlock::Text {
                text: text.to_string(),
            }],
            is_private: false,
            timestamp: ts,
            usage: None,
        }
    }

    #[test]
    fn empty_history_has_no_graph_content() {
        let graph = build_memory_graph(&[]);
        assert!(graph.is_empty());
        assert_eq!(recurrence_signal(&graph), None);
    }

    #[test]
    fn identical_repeated_messages_are_deduped_by_content_hash() {
        // from_chunk's canonical_id is a content hash, so the exact same
        // text twice collapses to the same node -- a real property of the
        // design, not a bug in this wiring.
        let messages = vec![text_message("hello", 1), text_message("hello", 2)];
        let graph = build_memory_graph(&messages);
        assert_eq!(graph.len(), 1);
    }

    #[test]
    fn distinct_messages_build_a_real_graph() {
        let messages = vec![
            text_message("first thought", 1),
            text_message("second thought", 2),
            text_message("third thought", 3),
        ];
        let graph = build_memory_graph(&messages);
        assert_eq!(graph.len(), 3);
    }

    #[test]
    fn empty_graph_has_no_dominant_glyph() {
        let graph = build_memory_graph(&[]);
        assert_eq!(dominant_glyph_byte(&graph), None);
    }

    #[test]
    fn dominant_glyph_is_deterministic_for_the_same_history() {
        let messages = vec![
            text_message("first thought", 1),
            text_message("second thought", 2),
        ];
        let a = dominant_glyph_byte(&build_memory_graph(&messages));
        let b = dominant_glyph_byte(&build_memory_graph(&messages));
        assert!(a.is_some());
        assert_eq!(
            a, b,
            "same conversation history must fold to the same glyph byte every time"
        );
    }
}
