//! Token-aware session compaction for the think context window.
//! Summarizes old messages, keeps recent N messages, merges nested summaries,
//! and extracts key files / pending work / timeline.

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
#[derive(Debug)]
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
            usage: None,
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

// ── Auto-Compaction ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompactionTrigger {
    MessageCount(usize),
    EnergyBelow(f64),
    TimeSecs(u64),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompactionStrategy {
    Micro,
    Session,
    Grouped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoCompactConfig {
    pub triggers: Vec<CompactionTrigger>,
    pub strategy: CompactionStrategy,
    pub last_compact_at: u64,
    pub compact_interval_secs: u64,
}

impl Default for AutoCompactConfig {
    fn default() -> Self {
        Self {
            triggers: vec![
                CompactionTrigger::MessageCount(50),
                CompactionTrigger::TimeSecs(3600),
            ],
            strategy: CompactionStrategy::Session,
            last_compact_at: 0,
            compact_interval_secs: 3600,
        }
    }
}

#[derive(Debug)]
pub struct MicroCompactEngine {
    pub batch_size: usize,
    pub keep_minimum: usize,
}

impl MicroCompactEngine {
    pub fn new(batch_size: usize, keep_minimum: usize) -> Self {
        Self {
            batch_size,
            keep_minimum,
        }
    }

    pub fn compact(&self, session: &mut Session) -> CompactionResult {
        let len = session.public_messages.len();
        if len <= self.keep_minimum {
            return CompactionResult::NotNeeded;
        }
        let removable = len - self.keep_minimum;
        let to_remove = self.batch_size.min(removable);
        if to_remove == 0 {
            return CompactionResult::NotNeeded;
        }
        session.public_messages.drain(..to_remove);
        CompactionResult::Compacted(CompactionSummary {
            key_files: Vec::new(),
            pending_items: Vec::new(),
            timeline: Vec::new(),
            compacted_count: to_remove,
            compacted_at: current_unix_timestamp(),
            narrative: format!("[MICRO-COMPACT: removed {} oldest messages]", to_remove),
        })
    }
}

#[derive(Debug)]
pub struct AutoCompactor {
    pub config: AutoCompactConfig,
    engine: CompactionEngine,
    micro_engine: MicroCompactEngine,
}

impl AutoCompactor {
    pub fn new(config: AutoCompactConfig) -> Self {
        Self {
            config,
            engine: CompactionEngine::default(),
            micro_engine: MicroCompactEngine::new(10, 5),
        }
    }

    pub fn should_compact(&self, session: &Session, energy_ratio: f64, now: u64) -> bool {
        for trigger in &self.config.triggers {
            match trigger {
                CompactionTrigger::MessageCount(n) => {
                    if session.public_messages.len() > *n {
                        return true;
                    }
                }
                CompactionTrigger::EnergyBelow(threshold) => {
                    if energy_ratio < *threshold {
                        return true;
                    }
                }
                CompactionTrigger::TimeSecs(secs) => {
                    if self.config.last_compact_at > 0
                        && now.saturating_sub(self.config.last_compact_at) >= *secs
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn compact_with_strategy(&mut self, session: &mut Session) -> CompactionResult {
        match self.config.strategy {
            CompactionStrategy::Micro => self.micro_engine.compact(session),
            CompactionStrategy::Session | CompactionStrategy::Grouped => {
                self.engine.compact(session)
            }
        }
    }

    pub fn compact_if_needed(
        &mut self,
        session: &mut Session,
        energy_ratio: f64,
    ) -> CompactionResult {
        let now = current_unix_timestamp();
        if self.should_compact(session, energy_ratio, now) {
            let result = self.compact_with_strategy(session);
            self.config.last_compact_at = now;
            result
        } else {
            CompactionResult::NotNeeded
        }
    }
}

// ── Token-aware compaction helpers ───────────────────────────────────────────

/// Preamble prepended to compact continuation messages.
pub const COMPACT_CONTINUATION_PREAMBLE: &str =
    "This session is being continued from a previous conversation that ran out of context. The summary below covers the earlier portion of the conversation.\n\n";

/// Instruction appended when `suppress_follow_up` is true.
pub const COMPACT_DIRECT_RESUME_INSTRUCTION: &str =
    "Continue the conversation from where it left off without asking the user any further questions. Resume directly — do not acknowledge the summary, do not recap what was happening, and do not preface with continuation text.";

/// Token estimation heuristic: 4 chars ≈ 1 token
pub fn estimate_message_tokens(msg: &ConversationMessage) -> usize {
    msg.blocks
        .iter()
        .map(|block| match block {
            ContentBlock::Text { text } => text.len() / 4 + 1,
            ContentBlock::ToolUse { name, input, .. } => (name.len() + input.len()) / 4 + 1,
            ContentBlock::ToolResult { output, .. } => output.len() / 4 + 1,
        })
        .sum()
}

/// Estimate total tokens for all public messages in a session.
pub fn estimate_session_tokens(session: &Session) -> usize {
    session
        .public_messages
        .iter()
        .map(estimate_message_tokens)
        .sum()
}

/// Returns true if the session should be compacted based on token count.
/// `max_tokens` — threshold above which compaction is triggered.
/// `preserve_recent` — number of most-recent messages excluded from the estimate.
pub fn should_compact_by_tokens(
    session: &Session,
    max_tokens: usize,
    preserve_recent: usize,
) -> bool {
    let msgs = &session.public_messages;
    let start = msgs.len().saturating_sub(preserve_recent);
    let compactable = &msgs[..start];
    compactable.len() > preserve_recent
        && compactable.iter().map(estimate_message_tokens).sum::<usize>() >= max_tokens
}

/// Format a continuation message from a compaction summary.
/// When `suppress_follow_up` is true the resume instruction is appended.
pub fn format_compact_continuation_message(summary: &str, suppress_follow_up: bool) -> String {
    let mut base = format!("{COMPACT_CONTINUATION_PREAMBLE}{summary}");
    if suppress_follow_up {
        base.push('\n');
        base.push_str(COMPACT_DIRECT_RESUME_INSTRUCTION);
    }
    base
}

/// Merge an optional existing summary with a new summary.
/// When `existing` is `Some`, the result includes labelled sections:
///   "Previously compacted context:" + "Newly compacted context:".
pub fn merge_compact_summaries(existing: Option<&str>, new_summary: &str) -> String {
    let Some(existing) = existing else {
        return new_summary.to_string();
    };

    let prev_highlights: Vec<&str> = existing
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    let new_highlights: Vec<&str> = new_summary
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();

    let mut lines = Vec::new();
    lines.push("Previously compacted context:".to_string());
    for l in &prev_highlights {
        lines.push(format!("  {l}"));
    }
    lines.push("Newly compacted context:".to_string());
    for l in &new_highlights {
        lines.push(format!("  {l}"));
    }
    lines.join("\n")
}

#[cfg(test)]
mod auto_compact_tests {
    use super::*;
    use crate::identity::AgentId;

    fn make_session(count: usize) -> Session {
        let id = AgentId::new("auto-compact-test");
        let mut session = Session::new(id, "auto-compact".to_string(), 0);
        for i in 0..count {
            session.public_messages.push(ConversationMessage::new_user(
                format!("Message {}", i),
                false,
            ));
        }
        session
    }

    #[test]
    fn test_micro_compact_removes_batch() {
        let mut session = make_session(20);
        let engine = MicroCompactEngine::new(5, 3);
        engine.compact(&mut session);
        assert_eq!(session.public_messages.len(), 15);
    }

    #[test]
    fn test_auto_compact_message_count_trigger() {
        let mut session = make_session(60);
        let config = AutoCompactConfig {
            triggers: vec![CompactionTrigger::MessageCount(50)],
            strategy: CompactionStrategy::Session,
            last_compact_at: 0,
            compact_interval_secs: 3600,
        };
        let mut compactor = AutoCompactor::new(config);
        let result = compactor.compact_if_needed(&mut session, 1.0);
        assert!(matches!(result, CompactionResult::Compacted(_)));
    }

    #[test]
    fn test_auto_compact_energy_trigger() {
        let mut session = make_session(10);
        let config = AutoCompactConfig {
            triggers: vec![CompactionTrigger::EnergyBelow(0.1)],
            strategy: CompactionStrategy::Micro,
            last_compact_at: 0,
            compact_interval_secs: 3600,
        };
        let mut compactor = AutoCompactor::new(config);
        // energy_ratio = 0.05 is below 0.1
        let triggered = compactor.should_compact(&session, 0.05, 0);
        assert!(triggered);
    }

    #[test]
    fn test_auto_compact_no_trigger() {
        let mut session = make_session(10);
        let config = AutoCompactConfig {
            triggers: vec![
                CompactionTrigger::MessageCount(50),
                CompactionTrigger::EnergyBelow(0.1),
            ],
            strategy: CompactionStrategy::Session,
            last_compact_at: 0,
            compact_interval_secs: 3600,
        };
        let mut compactor = AutoCompactor::new(config);
        let result = compactor.compact_if_needed(&mut session, 1.0);
        assert!(matches!(result, CompactionResult::NotNeeded));
    }

    #[test]
    fn test_auto_compact_updates_last_compact_at() {
        let mut session = make_session(60);
        let config = AutoCompactConfig {
            triggers: vec![CompactionTrigger::MessageCount(50)],
            strategy: CompactionStrategy::Session,
            last_compact_at: 0,
            compact_interval_secs: 3600,
        };
        let mut compactor = AutoCompactor::new(config);
        compactor.compact_if_needed(&mut session, 1.0);
        assert!(compactor.config.last_compact_at > 0);
    }
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

    // ── Token-aware compaction tests ──────────────────────────────────────────

    #[test]
    fn token_estimate_grows_with_content() {
        let short = ConversationMessage::new_user("hi".to_string(), false);
        let long = ConversationMessage::new_user("hello world ".repeat(100), false);
        assert!(
            estimate_message_tokens(&long) > estimate_message_tokens(&short),
            "longer message should have more estimated tokens"
        );
    }

    #[test]
    fn should_compact_by_tokens_triggers_when_over_limit() {
        // Make a session with enough tokens to trigger compaction
        let session = make_session_with_messages(20);
        // Very low max_tokens should trigger
        assert!(
            should_compact_by_tokens(&session, 1, 2),
            "should compact when token limit is very low"
        );
        // Very high max_tokens should not trigger
        assert!(
            !should_compact_by_tokens(&session, usize::MAX, 2),
            "should not compact when token limit is very high"
        );
    }

    #[test]
    fn continuation_message_contains_preamble() {
        let msg = format_compact_continuation_message("The agent worked on X.", true);
        assert!(
            msg.contains(COMPACT_CONTINUATION_PREAMBLE),
            "message should contain the preamble"
        );
        assert!(
            msg.contains(COMPACT_DIRECT_RESUME_INSTRUCTION),
            "message should contain the resume instruction when suppress_follow_up=true"
        );
    }

    #[test]
    fn merge_summaries_preserves_previous_context() {
        let merged = merge_compact_summaries(Some("old summary line"), "new summary line");
        assert!(
            merged.contains("Previously compacted context:"),
            "merged summary should label previous context"
        );
        assert!(
            merged.contains("Newly compacted context:"),
            "merged summary should label new context"
        );
        assert!(merged.contains("old summary line"));
        assert!(merged.contains("new summary line"));
    }
}
