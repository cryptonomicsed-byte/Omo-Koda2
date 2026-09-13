//! Per-call tool execution log — timing, previews, and stats.
//! Inspired by Core-Agency tool registry execution logging.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_MAX: usize = 200;
const PREVIEW_LEN: usize = 500;

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

/// A single tool invocation record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecution {
    pub tool: String,
    pub args_preview: String,
    pub result_preview: String,
    pub elapsed_ms: u64,
    pub at: u64,
}

/// Rolling execution log — last N tool calls with timing and previews.
#[derive(Debug)]
pub struct ExecutionLog {
    entries: VecDeque<ToolExecution>,
    max: usize,
}

impl ExecutionLog {
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            max: DEFAULT_MAX,
        }
    }

    #[must_use]
    pub fn with_capacity(max: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            max,
        }
    }

    /// Record a tool execution.
    pub fn log(&mut self, tool: &str, args: &str, result: &str, elapsed_ms: u64) {
        if self.entries.len() >= self.max {
            self.entries.pop_front();
        }
        self.entries.push_back(ToolExecution {
            tool: tool.to_string(),
            args_preview: truncate(args, PREVIEW_LEN),
            result_preview: truncate(result, PREVIEW_LEN),
            elapsed_ms,
            at: now_secs(),
        });
    }

    /// Most recent n entries (newest last).
    #[must_use]
    pub fn recent(&self, n: usize) -> Vec<&ToolExecution> {
        self.entries
            .iter()
            .rev()
            .take(n)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Summary string: call count, average latency, most-used tool.
    #[must_use]
    pub fn stats(&self) -> String {
        if self.entries.is_empty() {
            return "no tool calls recorded".to_string();
        }
        let total = self.entries.len();
        let avg_ms = self.entries.iter().map(|e| e.elapsed_ms).sum::<u64>() / total as u64;

        // Most frequent tool
        let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for e in &self.entries {
            *counts.entry(e.tool.as_str()).or_insert(0) += 1;
        }
        let top_tool = counts
            .into_iter()
            .max_by_key(|(_, c)| *c)
            .map(|(t, _)| t)
            .unwrap_or("?");

        format!("{} calls, avg {}ms, top tool: {}", total, avg_ms, top_tool)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for ExecutionLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_and_retrieve() {
        let mut log = ExecutionLog::new();
        log.log("bash", "ls /tmp", "file1\nfile2", 12);
        assert_eq!(log.len(), 1);
        let r = log.recent(1);
        assert_eq!(r[0].tool, "bash");
        assert_eq!(r[0].elapsed_ms, 12);
    }

    #[test]
    fn stats_reports_top_tool() {
        let mut log = ExecutionLog::new();
        log.log("bash", "a", "b", 10);
        log.log("bash", "c", "d", 20);
        log.log("read_file", "e", "f", 5);
        let s = log.stats();
        assert!(s.contains("3 calls"));
        assert!(s.contains("bash"));
    }
}
