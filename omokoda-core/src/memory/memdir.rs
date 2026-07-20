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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// Common English function words that would otherwise dominate word-overlap
/// scoring purely by frequency, before the vocabulary has seen enough
/// entries for IDF alone to suppress them. Not exhaustive -- just the
/// highest-frequency offenders.
const STOPWORDS: &[&str] = &[
    "that", "this", "with", "have", "from", "they", "will", "would", "could", "should", "about",
    "there", "their", "which", "when", "what", "were", "been", "your", "just", "like", "then",
    "than", "here", "some", "into", "over", "such", "only", "also", "very", "more", "most",
    "these", "those", "does", "each", "other", "because", "while",
];

/// Real, deterministic per-agent vocabulary learner -- no neural weights,
/// just document-frequency statistics that genuinely improve with data,
/// same honesty as `trade_outcome_learner.py`'s aggregation-not-ML pattern.
/// Tracks how many *distinct* odu_dir entries each word appears in, so
/// `recall`'s relevance scoring can weight a word by how informative it
/// actually is *for this specific agent's memory* -- a word she's said in
/// nearly every entry (or never at all yet) carries less signal than one
/// that shows up in just a few. This is what turns raw word-overlap into
/// something that structures itself over time as she accumulates memory.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WordLearner {
    /// word → number of live entries containing it at least once.
    doc_freq: HashMap<String, u32>,
    /// Total live entries the learner has indexed (mirrors `entries.len()`,
    /// kept separately so IDF math doesn't need a second borrow).
    doc_count: u32,
}

impl WordLearner {
    fn tokenize(text: &str) -> std::collections::HashSet<String> {
        text.split_whitespace()
            .map(|w| {
                w.trim_matches(|c: char| !c.is_alphanumeric())
                    .to_lowercase()
            })
            .filter(|w| w.len() >= 4 && !STOPWORDS.contains(&w.as_str()))
            .collect()
    }

    fn observe(&mut self, content: &str) {
        self.doc_count += 1;
        for word in Self::tokenize(content) {
            *self.doc_freq.entry(word).or_insert(0) += 1;
        }
    }

    fn forget(&mut self, content: &str) {
        self.doc_count = self.doc_count.saturating_sub(1);
        for word in Self::tokenize(content) {
            if let Some(count) = self.doc_freq.get_mut(&word) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    self.doc_freq.remove(&word);
                }
            }
        }
    }

    /// Inverse document frequency, smoothed so it's always positive and
    /// defined even before any entries exist. Rises the rarer a word is
    /// across this agent's own memory -- the actual "learning" signal.
    fn idf(&self, word: &str) -> f64 {
        let df = self.doc_freq.get(word).copied().unwrap_or(0) as f64;
        ((self.doc_count as f64 + 1.0) / (df + 1.0)).ln() + 1.0
    }
}

/// Cheap, real named-entity extraction -- no NLP model, just the same
/// heuristic class `compact.rs::looks_like_file_path` already uses for key
/// files: capitalized words (proper nouns -- names, projects, places) and
/// file-path-shaped tokens. This is what turns `odu_dir` from a bag of text
/// blobs into something with real, queryable structure: "what have I said
/// about Vantage" becomes an index lookup instead of hoping the substring
/// "vantage" shows up in the right place.
fn extract_entities(content: &str) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for raw_word in content.split_whitespace() {
        let word = raw_word
            .trim_matches(|c: char| !c.is_alphanumeric() && c != '/' && c != '.' && c != '_');
        if word.len() < 3 {
            continue;
        }
        let is_path = word.contains('/') || word.contains('.');
        let is_proper_noun = word
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
            && word.chars().skip(1).any(|c| c.is_lowercase());
        if !is_path && !is_proper_noun {
            continue;
        }
        let key = word.to_lowercase();
        if seen.insert(key) {
            out.push(word.to_string());
        }
    }
    out
}

/// Keyless structural commitment over a REM fold's micro entries -- the
/// same "verify without holding keys" pattern as zerolang's `gix1_valid`
/// example (GIX1 envelope auditing): a deterministic hash over the sorted
/// content hashes of every entry a fold claims to contain. `dream.rs`
/// tags a macro node with this at fold time; `OduDirectory::
/// verify_fold_integrity` recomputes it from whatever is *currently*
/// archived and compares. A mismatch means the archive was corrupted or
/// truncated independent of dream.rs's own folding logic -- a real, cheap
/// second check, not a repeat of the same code path that did the folding.
pub fn fold_commitment(entries: &[OduEntry]) -> String {
    use sha2::{Digest, Sha256};
    let mut hashes: Vec<[u8; 32]> = entries
        .iter()
        .map(|e| larql_glyph::content_hash(&e.content))
        .collect();
    hashes.sort_unstable();
    let mut hasher = Sha256::new();
    for h in &hashes {
        hasher.update(h);
    }
    hex::encode(hasher.finalize())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OduDirectory {
    pub entries: HashMap<String, OduEntry>,
    pub swarm_shared: HashMap<String, Vec<OduEntry>>,
    /// Fractal fold archive: macro-node id → the micro entries it compressed.
    /// Folds are lossless — a REM cycle moves noise clusters here instead of
    /// deleting them, and [`Self::unfold`] restores them on demand. This is
    /// the scale-invariance property: zoomed out, one macro node; zoomed in,
    /// the original sub-graph.
    pub archived_folds: HashMap<String, Vec<OduEntry>>,
    /// This agent's own evolving vocabulary statistics (see [`WordLearner`]).
    /// Updated on every insert/remove so `recall`'s relevance scoring
    /// structures itself around what she's actually talked about, not a
    /// fixed word list.
    #[serde(default)]
    vocab: WordLearner,
    /// Entity index: lowercased entity (see [`extract_entities`]) → entry
    /// ids that mention it. This is the structured half of recall --
    /// `recall_entity`/LARQL's `VERIFY WHERE entity = ...` are real
    /// traversals over this, not a hopeful substring search.
    #[serde(default)]
    entity_index: HashMap<String, Vec<String>>,
    /// Content-hash index (larql_glyph::content_hash — the same GIX-FOLD-v1
    /// digest divination.rs uses) → entry ids sharing that exact content.
    /// Observational only: unlike a cache, `insert` never auto-merges on a
    /// hash match. Real conversation turns can legitimately repeat verbatim
    /// ("hi" twice is two real turns, not noise), so silently collapsing
    /// them would corrupt the record. This index exists so a caller that
    /// *wants* dedup (e.g. a future dream.rs consolidation pass) can find
    /// exact-duplicate clusters explicitly, on purpose, without recall or
    /// insert ever doing it implicitly.
    #[serde(default, with = "hex_key_index")]
    content_hash_index: HashMap<[u8; 32], Vec<String>>,
}

/// Serde adapter for `content_hash_index`. serde_json cannot serialize a map
/// with `[u8; 32]` keys ("key must be a string"), which silently broke agent
/// auto-save the moment this index held any entry (e.g. after a private thought
/// or /seal). Persist the keys as hex strings; the values are unchanged.
mod hex_key_index {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::HashMap;

    pub fn serialize<S: Serializer>(
        map: &HashMap<[u8; 32], Vec<String>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let as_hex: HashMap<String, &Vec<String>> =
            map.iter().map(|(k, v)| (hex::encode(k), v)).collect();
        as_hex.serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<HashMap<[u8; 32], Vec<String>>, D::Error> {
        let as_hex = HashMap::<String, Vec<String>>::deserialize(deserializer)?;
        as_hex
            .into_iter()
            .map(|(k, v)| {
                let bytes = hex::decode(&k).map_err(serde::de::Error::custom)?;
                let arr: [u8; 32] = bytes.try_into().map_err(|_| {
                    serde::de::Error::custom("content_hash_index key must be 32 bytes")
                })?;
                Ok((arr, v))
            })
            .collect()
    }
}

impl OduDirectory {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            swarm_shared: HashMap::new(),
            archived_folds: HashMap::new(),
            vocab: WordLearner::default(),
            entity_index: HashMap::new(),
            content_hash_index: HashMap::new(),
        }
    }

    /// Exact-duplicate clusters currently live in the directory (>= 2
    /// entries sharing a content hash), each as `(hash, entry_ids)`. Purely
    /// observational -- see `content_hash_index`'s doc comment for why
    /// `insert` never acts on this automatically.
    pub fn duplicate_clusters(&self) -> Vec<(&[u8; 32], &Vec<String>)> {
        self.content_hash_index
            .iter()
            .filter(|(_, ids)| ids.len() >= 2)
            .collect()
    }

    /// Keyless structural audit of a REM fold (see [`fold_commitment`]):
    /// recomputes the commitment from whatever is currently archived under
    /// `macro_id` and compares it against the tag dream.rs stamped on the
    /// macro node at fold time. `Ok(())` only if the macro node exists,
    /// carries a `fold-integrity:` tag, an archive exists for it, and the
    /// recomputed commitment matches exactly.
    pub fn verify_fold_integrity(&self, macro_id: &str) -> Result<(), String> {
        let macro_entry = self
            .entries
            .get(macro_id)
            .ok_or_else(|| format!("no macro node '{macro_id}'"))?;
        let expected = macro_entry
            .tags
            .iter()
            .find_map(|t| t.strip_prefix("fold-integrity:"))
            .ok_or_else(|| format!("macro node '{macro_id}' carries no fold-integrity tag"))?;
        let archived = self
            .archived_folds
            .get(macro_id)
            .ok_or_else(|| format!("no archived fold for '{macro_id}'"))?;
        let actual = fold_commitment(archived);
        if actual == expected {
            Ok(())
        } else {
            Err(format!(
                "fold integrity mismatch for '{macro_id}': expected {expected}, got {actual}"
            ))
        }
    }

    fn index_entities(&mut self, entry_id: &str, content: &str) {
        for entity in extract_entities(content) {
            self.entity_index
                .entry(entity.to_lowercase())
                .or_default()
                .push(entry_id.to_string());
        }
    }

    fn unindex_entities(&mut self, entry_id: &str, content: &str) {
        for entity in extract_entities(content) {
            let key = entity.to_lowercase();
            if let Some(ids) = self.entity_index.get_mut(&key) {
                ids.retain(|id| id != entry_id);
                if ids.is_empty() {
                    self.entity_index.remove(&key);
                }
            }
        }
    }

    /// Real structured lookup: every live entry that mentions `entity`
    /// (case-insensitive), most-recently-touched first. Unlike `recall`,
    /// this is exact -- it answers "have I ever stored anything about
    /// exactly this name/path," not "what's semantically close."
    pub fn recall_entity(&self, entity: &str) -> Vec<&OduEntry> {
        let key = entity.to_lowercase();
        let Some(ids) = self.entity_index.get(&key) else {
            return Vec::new();
        };
        let mut hits: Vec<&OduEntry> = ids.iter().filter_map(|id| self.entries.get(id)).collect();
        hits.sort_by_key(|b| std::cmp::Reverse(b.last_accessed));
        hits
    }

    /// All entities this agent's memory currently indexes, for LARQL's
    /// `DESCRIBE entities` and for debugging/observability.
    pub fn known_entities(&self) -> Vec<&str> {
        self.entity_index.keys().map(|s| s.as_str()).collect()
    }

    pub fn insert(&mut self, entry: OduEntry) {
        // Re-inserting an existing id (e.g. a caller updating an entry in
        // place) must not double-count it in the vocabulary -- forget the
        // old content first, same as a real remove+insert would.
        if let Some(old_content) = self.entries.get(&entry.id).map(|old| old.content.clone()) {
            self.vocab.forget(&old_content);
            self.unindex_entities(&entry.id, &old_content);
            let old_hash = larql_glyph::content_hash(&old_content);
            if let Some(ids) = self.content_hash_index.get_mut(&old_hash) {
                ids.retain(|id| id != &entry.id);
                if ids.is_empty() {
                    self.content_hash_index.remove(&old_hash);
                }
            }
        }
        self.vocab.observe(&entry.content);
        self.index_entities(&entry.id, &entry.content);
        let hash = larql_glyph::content_hash(&entry.content);
        self.content_hash_index
            .entry(hash)
            .or_default()
            .push(entry.id.clone());
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
        let removed = self.entries.remove(id);
        if let Some(entry) = &removed {
            self.vocab.forget(&entry.content);
            self.unindex_entities(&entry.id, &entry.content);
            let hash = larql_glyph::content_hash(&entry.content);
            if let Some(ids) = self.content_hash_index.get_mut(&hash) {
                ids.retain(|i| i != id);
                if ids.is_empty() {
                    self.content_hash_index.remove(&hash);
                }
            }
        }
        removed
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
        if let Some(macro_entry) = self.entries.remove(macro_id) {
            self.vocab.forget(&macro_entry.content);
            self.unindex_entities(macro_id, &macro_entry.content);
            let hash = larql_glyph::content_hash(&macro_entry.content);
            if let Some(ids) = self.content_hash_index.get_mut(&hash) {
                ids.retain(|id| id != macro_id);
                if ids.is_empty() {
                    self.content_hash_index.remove(&hash);
                }
            }
        }
        let count = entries.len();
        for mut entry in entries {
            entry.touch();
            self.vocab.observe(&entry.content);
            self.index_entities(&entry.id, &entry.content);
            let hash = larql_glyph::content_hash(&entry.content);
            self.content_hash_index
                .entry(hash)
                .or_default()
                .push(entry.id.clone());
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

    // ── Recall ─────────────────────────────────────────────────────────────

    /// Real, targeted long-term recall: score every entry by IDF-weighted
    /// word overlap with `query` (see [`WordLearner`]) times importance,
    /// take the top `limit`, and return their content. A REM-folded macro
    /// node that scores as relevant is transparently unfolded first, so
    /// recall surfaces the original detail rather than the compressed
    /// "[REM fold] N entries" summary -- the whole point of folds being
    /// lossless.
    ///
    /// IDF-weighted overlap, not embeddings: this stack has no local
    /// embedding model, and shipping one just for recall is a much heavier
    /// dependency than this kernel needs. Weighting by how rare each word
    /// is *in this agent's own memory* (rather than flat overlap counts)
    /// is what makes this genuinely learn over time -- a word that shows
    /// up in nearly every entry (hers or generic filler) contributes
    /// almost nothing, while a word she's only used a handful of times
    /// dominates the score, exactly the direction real relevance should
    /// point as her vocabulary accumulates.
    pub fn recall(&mut self, query: &str, limit: usize) -> Vec<String> {
        let query_words = WordLearner::tokenize(query);
        if query_words.is_empty() {
            return Vec::new();
        }

        let vocab = &self.vocab;
        let mut scored: Vec<(String, f64)> = self
            .entries
            .values()
            .filter_map(|e| {
                let content_lower = e.content.to_lowercase();
                let weight: f64 = query_words
                    .iter()
                    .filter(|w| content_lower.contains(w.as_str()))
                    .map(|w| vocab.idf(w))
                    .sum();
                if weight <= 0.0 {
                    return None;
                }
                Some((e.id.clone(), weight * (1.0 + e.importance)))
            })
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);

        let mut out = Vec::with_capacity(scored.len());
        for (id, _) in scored {
            let is_fold = self
                .entries
                .get(&id)
                .map(|e| e.tags.iter().any(|t| t == "rem-fold"))
                .unwrap_or(false);
            if is_fold {
                if let Some(path) = self.entries.get(&id).map(|e| e.path.clone()) {
                    if self.unfold(&id).is_some() {
                        for restored in self.entries_at_path(&path) {
                            out.push(restored.content.clone());
                        }
                        continue;
                    }
                }
            }
            if let Some(e) = self.entries.get_mut(&id) {
                e.touch();
                out.push(e.content.clone());
            }
        }
        out
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

    // ── WordLearner / recall ─────────────────────────────────────────────

    fn content_entry(id: &str, content: &str, importance: f64) -> OduEntry {
        let mut e = OduEntry::new(id, content, "test/path");
        e.importance = importance;
        e
    }

    #[test]
    fn recall_finds_entries_matching_the_query() {
        let mut dir = OduDirectory::new();
        dir.insert(content_entry("e1", "the vantage database uses sqlite", 0.5));
        dir.insert(content_entry(
            "e2",
            "trading strategy backtest results",
            0.5,
        ));
        let hits = dir.recall("tell me about the vantage database", 5);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].contains("sqlite"));
    }

    #[test]
    fn recall_ranks_rare_words_above_common_ones() {
        let mut dir = OduDirectory::new();
        // "message" appears in every entry (common); "zangbeto" appears in
        // only one (rare). A query matching both should rank the entry
        // that also contains the rare word higher.
        dir.insert(content_entry("common1", "message about weather today", 0.5));
        dir.insert(content_entry("common2", "message about lunch plans", 0.5));
        dir.insert(content_entry("common3", "message about the weekend", 0.5));
        dir.insert(content_entry(
            "rare",
            "message about zangbeto receipts",
            0.5,
        ));

        let hits = dir.recall("message zangbeto", 1);
        assert_eq!(hits.len(), 1);
        assert!(
            hits[0].contains("zangbeto"),
            "the entry sharing the rare word should outrank entries only sharing the common word, got: {:?}",
            hits
        );
    }

    #[test]
    fn recall_returns_nothing_for_a_query_with_no_matches() {
        let mut dir = OduDirectory::new();
        dir.insert(content_entry("e1", "vantage database sqlite", 0.5));
        assert!(dir.recall("completely unrelated topic here", 5).is_empty());
    }

    #[test]
    fn vocab_forgets_when_entry_is_removed() {
        let mut dir = OduDirectory::new();
        dir.insert(content_entry("e1", "zangbeto receipts signing", 0.5));
        dir.insert(content_entry("e2", "unrelated weather report", 0.5));
        let idf_before = dir.vocab.idf("zangbeto");
        dir.remove("e1");
        let idf_after = dir.vocab.idf("zangbeto");
        assert!(
            idf_after > idf_before,
            "removing the only entry containing a word should make it rarer (higher IDF), got before={idf_before} after={idf_after}"
        );
    }

    #[test]
    fn recall_unfolds_a_relevant_rem_fold_and_restores_vocab() {
        let mut dir = OduDirectory::new();
        let mut folded = OduEntry::new(
            "rem:topic:1",
            "[REM fold] 2 entries on 'topic/zangbeto': zangbeto signing detail one | zangbeto signing detail two",
            "topic/zangbeto",
        );
        folded.importance = 0.5;
        folded.tags.push("rem-fold".to_string());
        dir.insert(folded);
        dir.archive_fold(
            "rem:topic:1",
            vec![
                OduEntry::new("m1", "zangbeto signing detail one", "topic/zangbeto"),
                OduEntry::new("m2", "zangbeto signing detail two", "topic/zangbeto"),
            ],
        );
        // The macro node's own generic content won't match "signing", but
        // the archived micro entries will once unfolded.
        let hits = dir.recall("zangbeto signing", 5);
        assert!(hits.iter().any(|c| c.contains("detail one")));
        assert!(hits.iter().any(|c| c.contains("detail two")));
        assert_eq!(
            dir.archived_fold_count(),
            0,
            "fold should have been consumed by unfold"
        );
    }
}
