//! Hash-keyed LLM response memoization with TTL and LRU eviction.
//! Inspired by Core-Agency neuralCache.ts (cyrb53 hash, 1hr TTL).

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_ENTRIES: usize = 100;
const TTL_SECS: u64 = 3600;

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// FNV-1a 64-bit hash of a string — fast, no-dep, collision-resistant enough for cache keys.
fn fnv1a(s: &str) -> u64 {
    let mut hash: u64 = 14695981039346656037;
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}

#[derive(Debug, Clone)]
struct CacheEntry {
    response: String,
    inserted_at: u64,
    last_used: u64,
}

/// LRU response cache keyed by FNV-1a hash of the prompt.
/// Max 100 entries, 1hr TTL. Thread-safe use: wrap in `Arc<Mutex<NeuralCache>>`.
#[derive(Debug, Default)]
pub struct NeuralCache {
    entries: HashMap<u64, CacheEntry>,
}

impl NeuralCache {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Look up a cached response for a prompt. Returns `None` if missing or expired.
    pub fn get(&mut self, prompt: &str) -> Option<String> {
        let key = fnv1a(prompt);
        let now = now_secs();
        if let Some(entry) = self.entries.get_mut(&key) {
            if now.saturating_sub(entry.inserted_at) < TTL_SECS {
                entry.last_used = now;
                return Some(entry.response.clone());
            }
            // Expired — remove
            self.entries.remove(&key);
        }
        None
    }

    /// Store a prompt → response pair. Prunes before inserting if at capacity.
    pub fn put(&mut self, prompt: &str, response: String) {
        if self.entries.len() >= MAX_ENTRIES {
            self.prune();
        }
        let now = now_secs();
        self.entries.insert(
            fnv1a(prompt),
            CacheEntry {
                response,
                inserted_at: now,
                last_used: now,
            },
        );
    }

    /// Remove expired entries, then LRU-evict down to 80% capacity.
    pub fn prune(&mut self) {
        let now = now_secs();
        self.entries
            .retain(|_, e| now.saturating_sub(e.inserted_at) < TTL_SECS);

        let target = MAX_ENTRIES * 4 / 5;
        if self.entries.len() > target {
            let mut pairs: Vec<(u64, u64)> = self
                .entries
                .iter()
                .map(|(k, e)| (*k, e.last_used))
                .collect();
            pairs.sort_unstable_by_key(|&(_, lu)| lu);
            let evict_count = self.entries.len() - target;
            for (key, _) in pairs.into_iter().take(evict_count) {
                self.entries.remove(&key);
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn miss_on_empty_cache() {
        let mut cache = NeuralCache::new();
        assert!(cache.get("hello world").is_none());
    }

    #[test]
    fn hit_after_put() {
        let mut cache = NeuralCache::new();
        cache.put("what is 2+2", "4".to_string());
        assert_eq!(cache.get("what is 2+2"), Some("4".to_string()));
    }

    #[test]
    fn different_prompts_are_independent() {
        let mut cache = NeuralCache::new();
        cache.put("prompt A", "response A".to_string());
        cache.put("prompt B", "response B".to_string());
        assert_eq!(cache.get("prompt A"), Some("response A".to_string()));
        assert_eq!(cache.get("prompt B"), Some("response B".to_string()));
        assert!(cache.get("prompt C").is_none());
    }
}
