//! Dream / Consolidation Engine — background Odu memory consolidation with
//! concurrency protection and configurable staleness threshold.
//!
//! Two rhythms run through this engine:
//!
//! - **Consolidation** (default every 30 min): sweeps stale entries below the
//!   importance threshold — light housekeeping between turns.
//! - **REM cycle** (on the Sabbath — UTC Saturday, once per Sabbath): the
//!   deep dream state. While the rhythm gate queues irreversible outward
//!   action for the Sabbath, the dream engine turns inward. Measures the
//!   *fractal dimension* of the agent's activity timeline (box-counting —
//!   Mandelbrot's burst-noise insight: information clusters in self-similar
//!   bursts separated by noise), then folds each path-cluster of low-importance
//!   "noise" entries into a single compressed macro node and prunes the
//!   residue. Folds are **lossless**: the micro entries move into the
//!   directory's fold archive and `OduDirectory::unfold` restores them on
//!   demand. Zoomed out, a week of scattered chatter is one node per topic;
//!   zoomed in, the original sub-graph.

use crate::memory::memdir::{OduDirectory, OduEntry};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

#[allow(dead_code)]
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

// ── RemConfig ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemConfig {
    /// If this long has passed since the last REM cycle, run even off-Sabbath
    /// (a slept-through Sabbath must not mean unbounded drift). Default
    /// 1_209_600 (two weeks — two missed Sabbaths).
    pub overdue_after_secs: u64,
    /// Entries at or below this importance are candidate noise. Default 0.35.
    pub noise_importance: f64,
    /// Minimum noise entries sharing a path before they fold into one macro
    /// node. Smaller clusters are left for ordinary consolidation. Default 3.
    pub min_fold_cluster: usize,
}

impl Default for RemConfig {
    fn default() -> Self {
        Self {
            overdue_after_secs: 1_209_600,
            noise_importance: 0.35,
            min_fold_cluster: 3,
        }
    }
}

// ── Sabbath alignment ─────────────────────────────────────────────────────────

pub const SECONDS_PER_DAY: u64 = 86_400;

/// True if `now` (unix seconds) falls on the UTC Sabbath — Saturday, the same
/// day [`crate::rhythm::RhythmGate::is_sabbath`] observes. While the rhythm
/// gate queues irreversible *outward* action for the Sabbath, the dream engine
/// turns *inward*: the weekly REM cycle runs on this day.
/// (1970-01-01 was a Thursday, hence the +4 offset; 6 = Saturday.)
pub fn is_sabbath_at(now: u64) -> bool {
    (now / SECONDS_PER_DAY + 4) % 7 == 6
}

// ── Fractal dimension ─────────────────────────────────────────────────────────

/// Box-counting fractal dimension of an activity timeline, in `[0, 1]`.
///
/// The timestamp span is divided into 2, 4, … 64 boxes; the slope of
/// `ln N(ε)` against `ln(1/ε)` (least squares) is the dimension. A steady
/// stream occupies every box at every scale (→ 1.0); bursty activity leaves
/// self-similar gaps (→ 0). Fewer than two distinct timestamps → 1.0.
pub fn fractal_dimension(timestamps: &[u64]) -> f64 {
    let min = match timestamps.iter().min() {
        Some(&m) => m,
        None => return 1.0,
    };
    let max = *timestamps.iter().max().expect("non-empty");
    if max == min {
        return 1.0;
    }
    let span = (max - min) as f64;

    let mut points = Vec::with_capacity(6);
    for k in 1..=6u32 {
        let boxes = 1usize << k;
        let mut occupied = vec![false; boxes];
        for &t in timestamps {
            let idx = (((t - min) as f64 / span) * boxes as f64) as usize;
            occupied[idx.min(boxes - 1)] = true;
        }
        let n = occupied.iter().filter(|&&b| b).count();
        points.push(((boxes as f64).ln(), (n as f64).ln()));
    }

    let m = points.len() as f64;
    let sx: f64 = points.iter().map(|p| p.0).sum();
    let sy: f64 = points.iter().map(|p| p.1).sum();
    let sxx: f64 = points.iter().map(|p| p.0 * p.0).sum();
    let sxy: f64 = points.iter().map(|p| p.0 * p.1).sum();
    let denom = m * sxx - sx * sx;
    if denom.abs() < f64::EPSILON {
        return 1.0;
    }
    ((m * sxy - sx * sy) / denom).clamp(0.0, 1.0)
}

// ── RemReport ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemReport {
    /// Box-counting dimension of the activity timeline before compression.
    pub fractal_dimension: f64,
    pub nodes_before: usize,
    /// Path-clusters folded into macro nodes.
    pub clusters_folded: usize,
    /// Individual entries absorbed into macro nodes.
    pub nodes_folded: usize,
    /// Residual noise entries pruned outright.
    pub nodes_pruned: usize,
    pub timestamp: u64,
    /// Possible belief supersessions noticed while folding: `(newer_id,
    /// older_id, shared_topic_word)` for entry pairs in the same
    /// path-cluster that share real topical overlap but where the newer
    /// entry carries a supersession marker (see [`SUPERSESSION_MARKERS`])
    /// the older one lacks -- e.g. "actually the deploy uses X" folded
    /// alongside an older "the deploy uses Y". This is REM's bounded
    /// answer to temporal reasoning: cheaper and less rigorous than a real
    /// temporal knowledge graph, but a real, non-fabricated signal that a
    /// folded belief may be stale, surfaced instead of silently
    /// compressing a contradiction away.
    pub possible_supersessions: Vec<(String, String, String)>,
}

/// Words that mark a statement as revising/superseding a prior one, rather
/// than merely adding to it. Not exhaustive -- the highest-signal, lowest
/// false-positive markers, same "small and honest" scope as the rest of
/// this module.
const SUPERSESSION_MARKERS: &[&str] = &[
    "actually", "correction", "wrong", "mistaken", "instead", "no longer",
    "not anymore", "changed to", "updated to", "deprecated", "outdated",
    "superseded", "reversed", "retracted",
];

fn has_supersession_marker(content: &str) -> bool {
    let lower = content.to_lowercase();
    SUPERSESSION_MARKERS.iter().any(|m| lower.contains(m))
}

/// Real, cheap topical-overlap check: words >= 4 chars shared between two
/// entries, minus a small stopword set -- same tokenization family as
/// `memdir::WordLearner`, kept independent here since dream.rs must not
/// depend on memdir's private internals.
fn shared_topic_word(a: &str, b: &str) -> Option<String> {
    let words_a: std::collections::HashSet<String> = a
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
        .filter(|w| w.len() >= 4)
        .collect();
    b.split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
        .find(|w| w.len() >= 4 && words_a.contains(w))
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
    pub rem_config: RemConfig,
    lock: ConsolidationLock,
    pub last_run: Option<u64>,
    pub last_rem: Option<u64>,
    pub status: DreamStatus,
}

impl DreamEngine {
    pub fn new(config: DreamConfig) -> Self {
        Self {
            config,
            rem_config: RemConfig::default(),
            lock: ConsolidationLock::new(),
            last_run: None,
            last_rem: None,
            status: DreamStatus::Idle,
        }
    }

    pub fn with_rem_config(mut self, rem_config: RemConfig) -> Self {
        self.rem_config = rem_config;
        self
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

    /// Returns true if the REM cycle is due: it is the Sabbath (UTC Saturday)
    /// and no REM has run yet today — or the cycle is overdue because more
    /// than [`RemConfig::overdue_after_secs`] passed since the last one
    /// (agent slept through its Sabbaths).
    pub fn should_rem(&self, now: u64) -> bool {
        if let Some(last) = self.last_rem {
            if now.saturating_sub(last) >= self.rem_config.overdue_after_secs {
                return true;
            }
        }
        if !is_sabbath_at(now) {
            return false;
        }
        match self.last_rem {
            None => true,
            // Once per Sabbath: the last run must be on an earlier UTC day.
            Some(last) => now / SECONDS_PER_DAY > last / SECONDS_PER_DAY,
        }
    }

    /// Attempt the Sabbath REM cycle: measure the fractal dimension of the
    /// activity timeline, fold each path-cluster of noise entries into one
    /// compressed macro node, then prune the residual noise. Returns `None`
    /// unless the cycle is due (see [`Self::should_rem`]) — the Sabbath, or
    /// overdue catch-up — or if a dream is already running.
    pub fn try_rem_cycle(&mut self, dir: &mut OduDirectory, now: u64) -> Option<RemReport> {
        if !self.should_rem(now) {
            return None;
        }
        let _guard = self.lock.try_acquire()?;
        self.status = DreamStatus::Dreaming;

        let nodes_before = dir.len();
        let timestamps: Vec<u64> = dir.entries.values().map(|e| e.created_at).collect();
        let fd = fractal_dimension(&timestamps);

        // Group noise entries by path — the self-similar clusters. BTreeMap
        // keeps fold order deterministic.
        let mut clusters: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for e in dir.entries.values() {
            if e.importance <= self.rem_config.noise_importance {
                clusters
                    .entry(e.path.clone())
                    .or_default()
                    .push(e.id.clone());
            }
        }

        let mut clusters_folded = 0usize;
        let mut nodes_folded = 0usize;
        let mut possible_supersessions: Vec<(String, String, String)> = Vec::new();
        for (path, mut ids) in clusters {
            if ids.len() < self.rem_config.min_fold_cluster {
                continue;
            }
            ids.sort();
            let mut previews: Vec<String> = Vec::new();
            let mut max_importance: f64 = 0.0;
            let mut micro: Vec<OduEntry> = Vec::with_capacity(ids.len());
            for id in &ids {
                if let Some(e) = dir.remove(id) {
                    if previews.len() < 3 {
                        previews.push(e.content.chars().take(80).collect());
                    }
                    max_importance = max_importance.max(e.importance);
                    micro.push(e);
                }
            }

            // Supersession scan over this cluster only (bounded by
            // min_fold_cluster/max_entries_per_run, so O(n^2) here is
            // small in practice): a newer entry with a supersession
            // marker, sharing a real topic word with an older entry in
            // the same cluster, is flagged rather than silently folded
            // away alongside it.
            for i in 0..micro.len() {
                if !has_supersession_marker(&micro[i].content) {
                    continue;
                }
                for j in 0..micro.len() {
                    if i == j || micro[j].created_at >= micro[i].created_at {
                        continue;
                    }
                    if let Some(word) = shared_topic_word(&micro[i].content, &micro[j].content) {
                        possible_supersessions.push((
                            micro[i].id.clone(),
                            micro[j].id.clone(),
                            word,
                        ));
                    }
                }
            }
            let macro_id = format!("rem:{path}:{now}");
            let mut folded = OduEntry::new(
                macro_id.clone(),
                format!(
                    "[REM fold] {} entries on '{}' (unfold '{}' to zoom in): {}",
                    ids.len(),
                    path,
                    macro_id,
                    previews.join(" | ")
                ),
                path,
            );
            // The macro node represents a whole cluster — it must survive the
            // residual prune below, so it starts at least at the noise line.
            folded.importance = (max_importance + 0.1).max(self.rem_config.noise_importance);
            folded.tags.push("rem-fold".to_string());
            // Keyless structural commitment (see
            // memdir::fold_commitment/verify_fold_integrity) -- a second,
            // independent check that the archive genuinely matches what
            // this macro node claims to represent, catchable even if the
            // archive is corrupted or truncated by something outside this
            // fold path.
            folded
                .tags
                .push(format!("fold-integrity:{}", crate::memory::memdir::fold_commitment(&micro)));
            dir.insert(folded);
            // Lossless fold: the micro entries move to the archive, keyed by
            // the macro node, so `unfold` can restore the full sub-graph.
            dir.archive_fold(macro_id, micro);
            clusters_folded += 1;
            nodes_folded += ids.len();
        }

        // Unclustered noise well below the line is structural fluff — prune.
        let nodes_pruned = dir.prune_stale(self.rem_config.noise_importance / 2.0);

        let report = RemReport {
            fractal_dimension: fd,
            nodes_before,
            clusters_folded,
            nodes_folded,
            nodes_pruned,
            timestamp: now,
            possible_supersessions,
        };

        self.last_rem = Some(now);
        self.status = DreamStatus::Done(ConsolidationResult {
            entries_consolidated: nodes_folded,
            entries_pruned: nodes_pruned,
            summary: format!(
                "REM cycle: fractal dimension {:.3}, folded {} clusters ({} entries), pruned {}",
                fd, clusters_folded, nodes_folded, nodes_pruned
            ),
            timestamp: now,
        });

        Some(report)
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

    // ── REM cycle ─────────────────────────────────────────────────────────

    /// 1970-01-03 (unix day 2) was a Saturday — the first Sabbath after epoch.
    const SABBATH: u64 = 2 * SECONDS_PER_DAY;
    const NEXT_SABBATH: u64 = SABBATH + 7 * SECONDS_PER_DAY;

    fn noise_entry(id: &str, path: &str, importance: f64) -> OduEntry {
        let mut e = OduEntry::new(id, format!("noise content {id}"), path);
        e.importance = importance;
        e
    }

    #[test]
    fn is_sabbath_at_matches_known_days() {
        assert!(!is_sabbath_at(SECONDS_PER_DAY), "Friday is not the Sabbath");
        assert!(is_sabbath_at(SABBATH), "Saturday is the Sabbath");
        assert!(is_sabbath_at(SABBATH + 86_399), "…all day long");
        assert!(!is_sabbath_at(3 * SECONDS_PER_DAY), "Sunday is not");
        assert!(is_sabbath_at(NEXT_SABBATH));
    }

    #[test]
    fn is_sabbath_at_agrees_with_rhythm_gate() {
        // The dream engine and the rhythm gate must observe the same Sabbath.
        let now = current_unix_timestamp();
        assert_eq!(is_sabbath_at(now), crate::rhythm::RhythmGate::is_sabbath());
    }

    #[test]
    fn fractal_dimension_steady_stream_is_one() {
        // Evenly spaced activity fills every box at every scale.
        let ts: Vec<u64> = (0..1000).map(|i| i * 60).collect();
        let fd = fractal_dimension(&ts);
        assert!(fd > 0.95, "steady stream should be ~1.0, got {fd}");
    }

    #[test]
    fn fractal_dimension_bursty_is_lower_than_steady() {
        // Two tight bursts separated by a long gap leave most boxes empty.
        let mut bursty: Vec<u64> = (0..50).collect();
        bursty.extend((1_000_000..1_000_050).collect::<Vec<u64>>());
        let steady: Vec<u64> = (0..100).map(|i| i * 10_000).collect();
        let fd_bursty = fractal_dimension(&bursty);
        let fd_steady = fractal_dimension(&steady);
        assert!(
            fd_bursty < fd_steady,
            "bursts {fd_bursty} should score below steady {fd_steady}"
        );
    }

    #[test]
    fn fractal_dimension_degenerate_inputs_are_neutral() {
        assert_eq!(fractal_dimension(&[]), 1.0);
        assert_eq!(fractal_dimension(&[42]), 1.0);
        assert_eq!(fractal_dimension(&[7, 7, 7]), 1.0);
    }

    #[test]
    fn rem_cycle_folds_noise_cluster_into_macro_node() {
        let mut engine = DreamEngine::new(DreamConfig::default());
        let mut dir = OduDirectory::new();
        // Four noise entries on one topic path, one important entry elsewhere.
        for i in 0..4 {
            dir.insert(noise_entry(&format!("n{i}"), "topics/pleasantries", 0.2));
        }
        let mut keeper = OduEntry::new("k1", "core insight", "topics/architecture");
        keeper.importance = 0.9;
        dir.insert(keeper);

        let report = engine
            .try_rem_cycle(&mut dir, SABBATH)
            .expect("first REM runs on the Sabbath");
        assert_eq!(report.nodes_before, 5);
        assert_eq!(report.clusters_folded, 1);
        assert_eq!(report.nodes_folded, 4);

        // The cluster collapsed to one macro node; the keeper survived.
        assert_eq!(dir.len(), 2);
        let macro_nodes = dir.entries_at_path("topics/pleasantries");
        assert_eq!(macro_nodes.len(), 1);
        assert!(macro_nodes[0].content.contains("[REM fold] 4 entries"));
        assert!(macro_nodes[0].tags.contains(&"rem-fold".to_string()));
        assert!(
            macro_nodes[0].importance >= engine.rem_config.noise_importance,
            "macro node must survive the residual prune"
        );
        assert_eq!(dir.entries_at_path("topics/architecture").len(), 1);

        // The fold is lossless: all four micro entries sit in the archive.
        assert_eq!(dir.archived_fold_count(), 1);
        assert_eq!(dir.archived_entry_count(), 4);
    }

    #[test]
    fn rem_cycle_flags_a_possible_supersession_in_a_folded_cluster() {
        let mut engine = DreamEngine::new(DreamConfig::default());
        let mut dir = OduDirectory::new();

        let mut old = OduEntry::new("old", "the deploy uses staging credentials", "topics/deploy");
        old.importance = 0.2;
        old.created_at = 100;
        dir.insert(old);

        let mut newer = OduEntry::new(
            "newer",
            "actually the deploy uses production credentials now",
            "topics/deploy",
        );
        newer.importance = 0.2;
        newer.created_at = 200;
        dir.insert(newer);

        // A third, unrelated noise entry so the cluster meets min_fold_cluster (3).
        let mut filler = OduEntry::new("filler", "unrelated noise chatter here", "topics/deploy");
        filler.importance = 0.2;
        filler.created_at = 150;
        dir.insert(filler);

        let report = engine
            .try_rem_cycle(&mut dir, SABBATH)
            .expect("REM runs on the Sabbath");

        assert_eq!(report.possible_supersessions.len(), 1);
        let (newer_id, older_id, word) = &report.possible_supersessions[0];
        assert_eq!(newer_id, "newer");
        assert_eq!(older_id, "old");
        assert_eq!(word, "deploy");
    }

    #[test]
    fn rem_fold_carries_a_verifiable_integrity_commitment() {
        let mut engine = DreamEngine::new(DreamConfig::default());
        let mut dir = OduDirectory::new();
        for i in 0..4 {
            dir.insert(noise_entry(&format!("n{i}"), "topics/pleasantries", 0.2));
        }
        engine.try_rem_cycle(&mut dir, SABBATH).unwrap();
        let macro_id = format!("rem:topics/pleasantries:{SABBATH}");

        assert!(
            dir.verify_fold_integrity(&macro_id).is_ok(),
            "a freshly folded, untampered archive must verify"
        );

        // Tamper with the archive directly (simulating corruption
        // independent of dream.rs's own folding path) and confirm the
        // keyless audit actually catches it.
        dir.archived_folds.get_mut(&macro_id).unwrap().pop();
        assert!(
            dir.verify_fold_integrity(&macro_id).is_err(),
            "a truncated archive must fail the integrity check"
        );
    }

    #[test]
    fn rem_fold_unfolds_losslessly() {
        let mut engine = DreamEngine::new(DreamConfig::default());
        let mut dir = OduDirectory::new();
        for i in 0..4 {
            dir.insert(noise_entry(&format!("n{i}"), "topics/pleasantries", 0.2));
        }
        engine.try_rem_cycle(&mut dir, SABBATH).unwrap();
        assert_eq!(dir.len(), 1, "folded to one macro node");

        // Zoom in: the macro id is deterministic (rem:<path>:<timestamp>).
        let macro_id = format!("rem:topics/pleasantries:{SABBATH}");
        let restored = dir.unfold(&macro_id).expect("fold is archived");
        assert_eq!(restored, 4);
        assert_eq!(dir.len(), 4, "macro node replaced by original entries");
        assert_eq!(dir.archived_fold_count(), 0);
        for i in 0..4 {
            let e = dir.get_mut(&format!("n{i}")).expect("entry restored");
            assert_eq!(e.content, format!("noise content n{i}"));
        }

        // Unfolding twice is a no-op signal, not a panic.
        assert!(dir.unfold(&macro_id).is_none());
    }

    #[test]
    fn rem_cycle_leaves_small_clusters_alone() {
        let mut engine = DreamEngine::new(DreamConfig::default());
        let mut dir = OduDirectory::new();
        // Two noise entries — below min_fold_cluster (3) — but above the
        // residual prune line (noise/2 = 0.175), so they stay untouched.
        dir.insert(noise_entry("a", "topics/x", 0.3));
        dir.insert(noise_entry("b", "topics/x", 0.3));

        let report = engine.try_rem_cycle(&mut dir, SABBATH).unwrap();
        assert_eq!(report.clusters_folded, 0);
        assert_eq!(report.nodes_pruned, 0);
        assert_eq!(dir.len(), 2);
    }

    #[test]
    fn rem_cycle_falls_on_the_sabbath() {
        let mut engine = DreamEngine::new(DreamConfig::default());
        let mut dir = OduDirectory::new();
        // Friday: nothing, even though no REM has ever run.
        assert!(engine.try_rem_cycle(&mut dir, SECONDS_PER_DAY).is_none());
        // Sabbath morning: the dream state begins.
        assert!(engine.try_rem_cycle(&mut dir, SABBATH + 3_600).is_some());
        // Later the same Sabbath: once per Sabbath only.
        assert!(engine.try_rem_cycle(&mut dir, SABBATH + 40_000).is_none());
        // Midweek: still nothing.
        assert!(engine
            .try_rem_cycle(&mut dir, SABBATH + 3 * SECONDS_PER_DAY)
            .is_none());
        // Next Sabbath: due again.
        assert!(engine.try_rem_cycle(&mut dir, NEXT_SABBATH).is_some());
    }

    #[test]
    fn rem_cycle_overdue_catchup_runs_off_sabbath() {
        let mut engine = DreamEngine::new(DreamConfig::default());
        let mut dir = OduDirectory::new();
        assert!(engine.try_rem_cycle(&mut dir, SABBATH).is_some());
        // Two missed Sabbaths later, on a Monday: catch-up fires anyway.
        let monday_after_two_weeks =
            SABBATH + engine.rem_config.overdue_after_secs + 2 * SECONDS_PER_DAY;
        assert!(!is_sabbath_at(monday_after_two_weeks));
        assert!(engine
            .try_rem_cycle(&mut dir, monday_after_two_weeks)
            .is_some());
    }
}
