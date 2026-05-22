//! Dream / Consolidation Engine — background Odu memory consolidation with
//! concurrency protection and configurable staleness threshold.

use crate::memory::memdir::OduDirectory;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

fn current_unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ── DreamConfig ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamConfig {
    /// How often consolidation runs (seconds). Default 1800 (30 min).
    pub consolidation_interval_secs: u64,
    /// Maximum entries processed per consolidation run. Default 50.
    pub max_entries_per_run: usize,
    /// Prompt template; `{entries}` is replaced with the entry text.
    pub prompt_template: String,
    /// Entries with importance below this are consolidated. Default 0.2.
    pub stale_threshold: f64,
}

impl Default for DreamConfig {
    fn default() -> Self {
        Self {
            consolidation_interval_secs: 1800,
            max_entries_per_run: 50,
            prompt_template:
                "Consolidate the following memory entries into a coherent summary:\n{entries}"
                    .to_string(),
            stale_threshold: 0.2,
        }
    }
}

// ── ConsolidationLock ─────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct ConsolidationLock {
    inner: Arc<Mutex<bool>>,
}

impl ConsolidationLock {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(false)),
        }
    }

    /// Try to acquire the lock. Returns `None` if already locked.
    pub fn try_acquire(&self) -> Option<ConsolidationGuard<'_>> {
        let mut guard = self.inner.lock().unwrap();
        if *guard {
            None
        } else {
            *guard = true;
            Some(ConsolidationGuard { lock: &self.inner })
        }
    }

    pub fn is_locked(&self) -> bool {
        *self.inner.lock().unwrap()
    }
}

impl Default for ConsolidationLock {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ConsolidationGuard<'a> {
    lock: &'a Arc<Mutex<bool>>,
}

impl<'a> Drop for ConsolidationGuard<'a> {
    fn drop(&mut self) {
        let mut guard = self.lock.lock().unwrap();
        *guard = false;
    }
}

// ── ConsolidationResult ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationResult {
    pub entries_consolidated: usize,
    pub entries_pruned: usize,
    pub summary: String,
    pub timestamp: u64,
}

// ── DreamStatus ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DreamStatus {
    Idle,
    Dreaming,
    Done(ConsolidationResult),
    Failed(String),
}

// ── DreamEngine ───────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct DreamEngine {
    pub config: DreamConfig,
    lock: ConsolidationLock,
    pub last_run: Option<u64>,
    pub status: DreamStatus,
}

impl DreamEngine {
    pub fn new(config: DreamConfig) -> Self {
        Self {
            config,
            lock: ConsolidationLock::new(),
            last_run: None,
            status: DreamStatus::Idle,
        }
    }

    /// Returns true if enough time has elapsed since the last run (or never run).
    pub fn should_consolidate(&self, now: u64) -> bool {
        match self.last_run {
            None => true,
            Some(last) => now.saturating_sub(last) >= self.config.consolidation_interval_secs,
        }
    }

    /// Attempt to run consolidation. Returns `None` if locked or interval not met.
    pub fn try_consolidate(
        &mut self,
        dir: &mut OduDirectory,
        now: u64,
    ) -> Option<ConsolidationResult> {
        if !self.should_consolidate(now) {
            return None;
        }

        // Try to acquire the lock; bail if already running.
        let _guard = self.lock.try_acquire()?;

        self.status = DreamStatus::Dreaming;

        // Decay entries by elapsed time since last run.
        let elapsed = match self.last_run {
            Some(last) => now.saturating_sub(last),
            None => 0,
        };
        dir.age_entries(elapsed);

        // Collect stale entry ids, capped at max_entries_per_run.
        let stale_ids: Vec<String> = dir
            .scan_stale(self.config.stale_threshold)
            .into_iter()
            .take(self.config.max_entries_per_run)
            .collect();

        let entries_consolidated = stale_ids.len();

        // Remove stale entries and collect content.
        let mut content_parts: Vec<String> = Vec::with_capacity(entries_consolidated);
        for id in &stale_ids {
            if let Some(entry) = dir.remove(id) {
                content_parts.push(format!("[{}] {}", entry.path, entry.content));
            }
        }

        // Build summary.
        let topic_preview: Vec<&str> = content_parts.iter().take(5).map(|s| s.as_str()).collect();
        let summary = format!(
            "Consolidated {} memory entries. Topics: {}",
            entries_consolidated,
            topic_preview.join("; ")
        );

        // Prune remaining stale entries at half the threshold.
        let entries_pruned = dir.prune_stale(self.config.stale_threshold / 2.0);

        let result = ConsolidationResult {
            entries_consolidated,
            entries_pruned,
            summary,
            timestamp: now,
        };

        self.last_run = Some(now);
        self.status = DreamStatus::Done(result.clone());

        // _guard is dropped here, releasing the lock.
        Some(result)
    }

    pub fn is_dreaming(&self) -> bool {
        self.lock.is_locked()
    }

    /// Replace `{entries}` in the prompt template with the joined entry strings.
    pub fn build_prompt(&self, entries: &[String]) -> String {
        self.config
            .prompt_template
            .replace("{entries}", &entries.join("\n"))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod dream_tests {
    use super::*;
    use crate::memory::memdir::OduEntry;

    fn stale_entry(id: &str) -> OduEntry {
        let mut e = OduEntry::new(id, "some content", "test/path");
        e.importance = 0.05;
        e
    }

    #[test]
    fn test_should_consolidate_first_run() {
        let engine = DreamEngine::new(DreamConfig::default());
        assert!(engine.should_consolidate(0));
    }

    #[test]
    fn test_should_consolidate_too_soon() {
        let mut engine = DreamEngine::new(DreamConfig::default());
        engine.last_run = Some(100);
        // 200 - 100 = 100 < 1800
        assert!(!engine.should_consolidate(200));
    }

    #[test]
    fn test_consolidation_lock_prevents_reentry() {
        let lock = ConsolidationLock::new();
        let _guard = lock.try_acquire().expect("first acquire should succeed");
        assert!(
            lock.try_acquire().is_none(),
            "second acquire should return None"
        );
    }

    #[test]
    fn test_try_consolidate_runs_and_updates() {
        let config = DreamConfig {
            consolidation_interval_secs: 0, // always ready
            stale_threshold: 0.2,
            max_entries_per_run: 50,
            ..DreamConfig::default()
        };
        let mut engine = DreamEngine::new(config);
        let mut dir = OduDirectory::new();
        dir.insert(stale_entry("e1"));
        dir.insert(stale_entry("e2"));
        dir.insert(stale_entry("e3"));

        let result = engine
            .try_consolidate(&mut dir, 1000)
            .expect("should consolidate");
        assert_eq!(result.entries_consolidated, 3);
        assert!(dir.len() == 0 || dir.len() < 3);
    }

    #[test]
    fn test_build_prompt() {
        let config = DreamConfig {
            prompt_template: "Summarize: {entries}".to_string(),
            ..DreamConfig::default()
        };
        let engine = DreamEngine::new(config);
        let result = engine.build_prompt(&["a".to_string(), "b".to_string()]);
        assert_eq!(result, "Summarize: a\nb");
    }

    #[test]
    fn test_dream_engine_idle_status() {
        let engine = DreamEngine::new(DreamConfig::default());
        assert!(matches!(engine.status, DreamStatus::Idle));
    }
}
