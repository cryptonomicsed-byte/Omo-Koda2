//! Living Odu Directory — hierarchical memory store with importance decay,
//! stale scanning, and swarm-shared paths.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn current_unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub const DECAY_PER_DAY: f64 = 0.10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OduEntry {
    pub id: String,
    pub content: String,
    pub importance: f64,
    pub created_at: u64,
    pub last_accessed: u64,
    pub tags: Vec<String>,
    pub path: String,
}

impl OduEntry {
    pub fn new(id: impl Into<String>, content: impl Into<String>, path: impl Into<String>) -> Self {
        let now = current_unix_timestamp();
        Self {
            id: id.into(),
            content: content.into(),
            importance: 0.5,
            created_at: now,
            last_accessed: now,
            tags: Vec::new(),
            path: path.into(),
        }
    }

    pub fn age_secs(&self, now: u64) -> u64 {
        now.saturating_sub(self.last_accessed)
    }

    pub fn touch(&mut self) {
        self.last_accessed = current_unix_timestamp();
    }
}

#[derive(Debug)]
pub struct OduDirectory {
    pub entries: HashMap<String, OduEntry>,
    pub swarm_shared: HashMap<String, Vec<OduEntry>>,
    /// Fractal fold archive: macro-node id → the micro entries it compressed.
    /// Folds are lossless — a REM cycle moves noise clusters here instead of
    /// deleting them, and [`Self::unfold`] restores them on demand. This is
    /// the scale-invariance property: zoomed out, one macro node; zoomed in,
    /// the original sub-graph.
    pub archived_folds: HashMap<String, Vec<OduEntry>>,
}

impl OduDirectory {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            swarm_shared: HashMap::new(),
            archived_folds: HashMap::new(),
        }
    }

    pub fn insert(&mut self, entry: OduEntry) {
        self.entries.insert(entry.id.clone(), entry);
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut OduEntry> {
        if let Some(entry) = self.entries.get_mut(id) {
            entry.touch();
            Some(entry)
        } else {
            None
        }
    }

    pub fn remove(&mut self, id: &str) -> Option<OduEntry> {
        self.entries.remove(id)
    }

    pub fn entries_at_path(&self, path: &str) -> Vec<&OduEntry> {
        let prefix = format!("{}/", path);
        self.entries
            .values()
            .filter(|e| e.path == path || e.path.starts_with(&prefix))
            .collect()
    }

    pub fn age_entries(&mut self, elapsed_secs: u64) {
        let elapsed_days = elapsed_secs as f64 / 86400.0;
        let decay = DECAY_PER_DAY * elapsed_days;
        for entry in self.entries.values_mut() {
            entry.importance = (entry.importance - decay).max(0.0);
        }
    }

    pub fn scan_stale(&self, min_importance: f64) -> Vec<String> {
        self.entries
            .values()
            .filter(|e| e.importance < min_importance)
            .map(|e| e.id.clone())
            .collect()
    }

    pub fn prune_stale(&mut self, min_importance: f64) -> usize {
        let stale_ids = self.scan_stale(min_importance);
        let count = stale_ids.len();
        for id in &stale_ids {
            self.entries.remove(id);
        }
        count
    }

    pub fn share_to_swarm(&mut self, agent_id: &str, entry: OduEntry) {
        self.swarm_shared
            .entry(agent_id.to_string())
            .or_default()
            .push(entry);
    }

    pub fn swarm_entries(&self, agent_id: &str) -> &[OduEntry] {
        self.swarm_shared
            .get(agent_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn swarm_agents(&self) -> Vec<&str> {
        self.swarm_shared.keys().map(|k| k.as_str()).collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    // ── Fractal fold archive ──────────────────────────────────────────────

    /// Archive the micro entries a REM fold compressed under `macro_id`.
    pub fn archive_fold(&mut self, macro_id: impl Into<String>, entries: Vec<OduEntry>) {
        if !entries.is_empty() {
            self.archived_folds.insert(macro_id.into(), entries);
        }
    }

    /// Zoom in: restore a fold's micro entries into the live directory and
    /// remove the macro node. Restored entries are touched (fresh
    /// `last_accessed`) but keep their original importance — if they are
    /// still noise, the next REM cycle folds them again. Returns the number
    /// of entries restored, or `None` if `macro_id` has no archived fold.
    pub fn unfold(&mut self, macro_id: &str) -> Option<usize> {
        let entries = self.archived_folds.remove(macro_id)?;
        self.entries.remove(macro_id);
        let count = entries.len();
        for mut entry in entries {
            entry.touch();
            self.entries.insert(entry.id.clone(), entry);
        }
        Some(count)
    }

    /// Number of archived folds (macro nodes with recoverable micro entries).
    pub fn archived_fold_count(&self) -> usize {
        self.archived_folds.len()
    }

    /// Total micro entries held across all archived folds.
    pub fn archived_entry_count(&self) -> usize {
        self.archived_folds.values().map(Vec::len).sum()
    }
}

impl Default for OduDirectory {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub total_entries: usize,
    pub stale_entries: usize,
    pub swarm_entries: usize,
    pub avg_importance: f64,
    pub oldest_entry_secs: u64,
}

pub struct MemoryScanner;

impl MemoryScanner {
    pub fn scan(dir: &OduDirectory) -> ScanResult {
        let now = current_unix_timestamp();
        let total_entries = dir.entries.len();
        let stale_entries = dir.entries.values().filter(|e| e.importance < 0.2).count();
        let swarm_entries: usize = dir.swarm_shared.values().map(|v| v.len()).sum();
        let avg_importance = if total_entries > 0 {
            dir.entries.values().map(|e| e.importance).sum::<f64>() / total_entries as f64
        } else {
            0.0
        };
        let oldest_entry_secs = dir
            .entries
            .values()
            .map(|e| e.age_secs(now))
            .max()
            .unwrap_or(0);

        ScanResult {
            total_entries,
            stale_entries,
            swarm_entries,
            avg_importance,
            oldest_entry_secs,
        }
    }

    pub fn team_path(team_name: &str) -> String {
        format!("team/{}", team_name)
    }

    pub fn swarm_path(agent_id: &str) -> String {
        format!("swarm/{}", agent_id)
    }
}

#[cfg(test)]
mod memdir_tests {
    use super::*;

    fn make_entry(id: &str, importance: f64) -> OduEntry {
        let mut e = OduEntry::new(id, "test content", "test/path");
        e.importance = importance;
        e
    }

    #[test]
    fn test_insert_and_get() {
        let mut dir = OduDirectory::new();
        dir.insert(make_entry("e1", 0.5));
        assert!(dir.get_mut("e1").is_some());
    }

    #[test]
    fn test_stale_scan() {
        let mut dir = OduDirectory::new();
        dir.insert(make_entry("stale", 0.05));
        let stale = dir.scan_stale(0.1);
        assert!(stale.contains(&"stale".to_string()));
    }

    #[test]
    fn test_age_decay() {
        let mut dir = OduDirectory::new();
        dir.insert(make_entry("e1", 1.0));
        dir.age_entries(86400); // one day
        let entry = dir.entries.get("e1").unwrap();
        let diff = (entry.importance - 0.9).abs();
        assert!(diff < 1e-9, "expected ~0.9, got {}", entry.importance);
    }

    #[test]
    fn test_prune_stale() {
        let mut dir = OduDirectory::new();
        dir.insert(make_entry("stale1", 0.05));
        dir.insert(make_entry("stale2", 0.05));
        dir.insert(make_entry("healthy", 0.8));
        let pruned = dir.prune_stale(0.1);
        assert_eq!(pruned, 2);
        assert_eq!(dir.len(), 1);
    }

    #[test]
    fn test_swarm_share() {
        let mut dir = OduDirectory::new();
        dir.share_to_swarm("agent-1", make_entry("e1", 0.5));
        dir.share_to_swarm("agent-1", make_entry("e2", 0.5));
        assert_eq!(dir.swarm_entries("agent-1").len(), 2);
    }

    #[test]
    fn test_scanner() {
        let mut dir = OduDirectory::new();
        dir.insert(make_entry("e1", 0.8));
        dir.insert(make_entry("e2", 0.1));
        dir.share_to_swarm("agent-1", make_entry("s1", 0.5));
        let result = MemoryScanner::scan(&dir);
        assert_eq!(result.total_entries, 2);
        assert_eq!(result.stale_entries, 1);
        assert_eq!(result.swarm_entries, 1);
    }

    #[test]
    fn test_team_path() {
        assert_eq!(MemoryScanner::team_path("orisha"), "team/orisha");
    }
}
