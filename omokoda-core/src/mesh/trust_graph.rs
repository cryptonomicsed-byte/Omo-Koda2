use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct TrustEdge {
    pub score: f64,
    pub last_updated: u64,
    pub interaction_count: u64,
}

impl TrustEdge {
    fn new(initial_score: f64) -> Self {
        Self {
            score: initial_score.clamp(0.0, 1.0),
            last_updated: now_secs(),
            interaction_count: 1,
        }
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub struct TrustGraph {
    edges: HashMap<String, HashMap<String, TrustEdge>>,
}

impl TrustGraph {
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
        }
    }

    pub fn update(&mut self, from: &str, to: &str, delta: f64) {
        let neighbor_map = self.edges.entry(from.to_string()).or_default();
        let entry = neighbor_map
            .entry(to.to_string())
            .or_insert_with(|| TrustEdge::new(0.5));
        entry.score = (entry.score + delta).clamp(0.0, 1.0);
        entry.last_updated = now_secs();
        entry.interaction_count += 1;
    }

    pub fn score(&self, from: &str, to: &str) -> f64 {
        self.edges
            .get(from)
            .and_then(|m| m.get(to))
            .map(|e| e.score)
            .unwrap_or(0.5)
    }

    pub fn neighbors(&self, agent_id: &str) -> Vec<String> {
        self.edges
            .get(agent_id)
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default()
    }

    pub fn below_threshold(&self, agent_id: &str, threshold: f64) -> Vec<String> {
        self.edges
            .get(agent_id)
            .map(|m| {
                m.iter()
                    .filter(|(_, e)| e.score < threshold)
                    .map(|(id, _)| id.clone())
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Default for TrustGraph {
    fn default() -> Self {
        Self::new()
    }
}
