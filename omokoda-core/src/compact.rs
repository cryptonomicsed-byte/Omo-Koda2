//! Session compaction — compress old messages into a summary to prevent context window overflow.
//!
//! Ports Claw-code's compact.rs pattern: summarize old messages, keep recent N messages,
//! merge nested summaries, extract key files / pending work / timeline.

use crate::session::{ContentBlock, ConversationMessage, MessageRole, Session};
use serde::{Deserialize, Serialize};

/// Result of a compaction operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompactionResult {
    NotNeeded,
    Compacted(CompactionSummary),
}

/// Extracted metadata from compacted messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionSummary {
    /// Files referenced in the compacted messages
    pub key_files: Vec<String>,
    /// Pending work items extracted from messages
    pub pending_items: Vec<String>,
    /// Brief timeline of key events
    pub timeline: Vec<TimelineEvent>,
    /// Number of messages that were compacted
    pub compacted_count: usize,
    /// Unix timestamp when compaction occurred
    pub compacted_at: u64,
    /// Narrative summary text
    pub narrative: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub timestamp: u64,
    pub description: String,
}

/// Engine that performs session compaction
pub struct CompactionEngine {
    /// Minimum messages before compaction is considered
    pub threshold: usize,
    /// Number of recent messages to keep after compaction
    pub keep_recent: usize,
}

impl Default for CompactionEngine {
    fn default() -> Self {
        Self {
            threshold: 50,
            keep_recent: 10,
        }
    }
}

impl CompactionEngine {
    pub fn new(threshold: usize, keep_recent: usize) -> Self {
        Self {
            threshold,
            keep_recent,
        }
    }

    /// Check if the session needs compaction
    pub fn needs_compaction(&self, session: &Session) -> bool {
        session.public_messages.len() > self.threshold
    }

    /// Compact the session: extract summary from old messages, keep recent N.
    /// Returns CompactionResult — if Compacted, the session has been modified in place.
    pub fn compact(&self, session: &mut Session) -> CompactionResult {
        if !self.needs_compaction(session) {
            return CompactionResult::NotNeeded;
        }

        let total = session.public_messages.len();
        let to_compact_count = total.saturating_sub(self.keep_recent);

        if to_compact_count == 0 {
            return CompactionResult::NotNeeded;
        }

        // Drain the old messages to be compacted
        let old_messages: Vec<ConversationMessage> =
            session.public_messages.drain(..to_compact_count).collect();

        let summary = self.extract_summary(&old_messages);

        // Insert a System summary message at the front of the remaining messages
        let summary_text = format_summary_text(&summary);
        let summary_message = ConversationMessage {
            role: MessageRole::System,
            blocks: vec![ContentBlock::Text { text: summary_text }],
            is_private: false,
            timestamp: summary.compacted_at,
        };
        session.public_messages.insert(0, summary_message);

        CompactionResult::Compacted(summary)
    }

    /// Extract key information from a set of messages
    fn extract_summary(&self, messages: &[ConversationMessage]) -> CompactionSummary {
        let now = current_unix_timestamp();
        let mut key_files = std::collections::HashSet::new();
        let mut pending_items = Vec::new();
        let mut timeline = Vec::new();
        let mut all_text = String::new();

        for msg in messages {
            let text = message_text_content(msg);
            if text.is_empty() {
                continue;
            }

            all_text.push_str(&text);
            all_text.push('\n');

            // Extract file references (paths with extensions)
            for word in text.split_whitespace() {
                if looks_like_file_path(word) {
                    key_files.insert(
                        word.trim_matches(|c: char| {
                            !c.is_alphanumeric() && c != '/' && c != '.' && c != '_' && c != '-'
                        })
                        .to_string(),
                    );
                }
            }

            // Extract pending work (lines starting with TODO/FIXME/- [ ])
            for line in text.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("TODO")
                    || trimmed.starts_with("FIXME")
                    || trimmed.starts_with("- [ ]")
                    || trimmed.starts_with("* [ ]")
                {
                    pending_items.push(trimmed.to_string());
                }
            }

            // Timeline: tool uses and significant events
            for block in &msg.blocks {
                if let ContentBlock::ToolUse { name, .. } = block {
                    timeline.push(TimelineEvent {
                        timestamp: msg.timestamp,
                        description: format!("Used tool: {}", name),
                    });
                }
            }
        }

        // Build narrative
        let word_count = all_text.split_whitespace().count();
        let narrative = if word_count > 200 {
            format!(
                "Session context ({} messages, ~{} words): Agent worked on tasks including tools: {}. Key files: {}.",
                messages.len(),
                word_count,
                timeline
                    .iter()
                    .map(|t| t.description.as_str())
                    .take(5)
                    .collect::<Vec<_>>()
                    .join(", "),
                key_files
                    .iter()
                    .take(10)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        } else {
            all_text.chars().take(500).collect()
        };

        CompactionSummary {
            key_files: key_files.into_iter().take(20).collect(),
            pending_items: pending_items.into_iter().take(10).collect(),
            timeline: timeline.into_iter().take(20).collect(),
            compacted_count: messages.len(),
            compacted_at: now,
            narrative,
        }
    }
}

fn message_text_content(msg: &ConversationMessage) -> String {
    msg.blocks
        .iter()
        .filter_map(|b| match b {
            ContentBlock::Text { text } => Some(text.as_str()),
            ContentBlock::ToolResult { output, .. } => Some(output.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn looks_like_file_path(s: &str) -> bool {
    let s = s.trim_matches(|c: char| !c.is_alphanumeric() && c != '/' && c != '.' && c != '_');
    s.contains('.')
        && (s.ends_with(".rs")
            || s.ends_with(".toml")
            || s.ends_with(".json")
            || s.ends_with(".ts")
            || s.ends_with(".tsx")
            || s.ends_with(".py")
            || s.ends_with(".md")
            || s.ends_with(".ex")
            || s.ends_with(".move")
            || s.contains('/'))
}

fn format_summary_text(summary: &CompactionSummary) -> String {
    let mut parts = vec![
        format!(
            "[COMPACTED CONTEXT: {} messages summarized]",
            summary.compacted_count
        ),
        summary.narrative.clone(),
    ];
    if !summary.key_files.is_empty() {
        parts.push(format!("Key files: {}", summary.key_files.join(", ")));
    }
    if !summary.pending_items.is_empty() {
        parts.push(format!(
            "Pending work:\n{}",
            summary.pending_items.join("\n")
        ));
    }
    parts.join("\n\n")
}

fn current_unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::AgentId;
    use crate::session::Session;

    fn make_session_with_messages(count: usize) -> Session {
        let id = AgentId::new("test-fingerprint-1234567890abcdef");
        let mut session = Session::new(id, "test".to_string(), 0);
        for i in 0..count {
            session.public_messages.push(ConversationMessage::new_user(
                format!("Message {}: Hello world", i),
                false,
            ));
        }
        session
    }

    #[test]
    fn test_compaction_not_needed() {
        let engine = CompactionEngine::new(50, 10);
        let mut session = make_session_with_messages(30);
        let result = engine.compact(&mut session);
        assert!(matches!(result, CompactionResult::NotNeeded));
        assert_eq!(session.public_messages.len(), 30);
    }

    #[test]
    fn test_compaction_triggered() {
        let engine = CompactionEngine::new(50, 10);
        let mut session = make_session_with_messages(60);
        let result = engine.compact(&mut session);
        assert!(matches!(result, CompactionResult::Compacted(_)));
        // Should keep 10 recent + 1 summary message at front
        assert_eq!(session.public_messages.len(), 11);
        // First message should be the System summary
        assert_eq!(session.public_messages[0].role, MessageRole::System);
    }

    #[test]
    fn test_needs_compaction() {
        let engine = CompactionEngine::new(50, 10);
        let session_small = make_session_with_messages(49);
        let session_large = make_session_with_messages(51);
        assert!(!engine.needs_compaction(&session_small));
        assert!(engine.needs_compaction(&session_large));
    }
}
