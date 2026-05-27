//! SOMA — Self-Organizing Memory Architecture (from Droidclaw).
//! MemCells: emotionally weighted memory atoms.
//! MemScenes: psychological theme clusters.
//! LPM: Lifelong Personal Model — the agent's persistent self-model.

use serde::{Deserialize, Serialize};

/// A single memory atom with emotional weight and activation count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemCell {
    pub content: String,
    pub tension: f64,
    pub connection_depth: f64,
    pub activation_count: u32,
    pub timestamp: u64,
}

impl MemCell {
    pub fn new(content: String, timestamp: u64) -> Self {
        Self {
            content,
            tension: 0.0,
            connection_depth: 0.0,
            activation_count: 0,
            timestamp,
        }
    }

    pub fn activate(&mut self) {
        self.activation_count += 1;
    }

    pub fn apply_tension(&mut self, delta: f64) {
        self.tension = (self.tension + delta).clamp(0.0, 1.0);
    }

    pub fn emotional_weight(&self) -> f64 {
        (self.tension * 0.4
            + self.connection_depth * 0.4
            + (self.activation_count as f64 * 0.01).min(0.2))
        .min(1.0)
    }
}

/// A thematic cluster of MemCells sharing psychological relevance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemScene {
    pub theme: String,
    pub cells: Vec<MemCell>,
    pub salience: f64,
}

impl MemScene {
    pub fn new(theme: String) -> Self {
        Self {
            theme,
            cells: Vec::new(),
            salience: 0.0,
        }
    }

    pub fn add_cell(&mut self, cell: MemCell) {
        self.salience += cell.emotional_weight() * 0.1;
        self.cells.push(cell);
    }

    pub fn most_salient(&self) -> Option<&MemCell> {
        self.cells.iter().max_by(|a, b| {
            a.emotional_weight()
                .partial_cmp(&b.emotional_weight())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

/// Lifelong Personal Model — the agent's persistent self-understanding.
/// Never derived from public data. Internal continuity substrate.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Lpm {
    pub scenes: Vec<MemScene>,
    pub total_activations: u64,
    pub peak_tension: f64,
}

impl Lpm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_scene(&mut self, scene: MemScene) {
        self.scenes.push(scene);
    }

    pub fn record_activation(&mut self, tension_delta: f64) {
        self.total_activations += 1;
        self.peak_tension = self.peak_tension.max(tension_delta);
    }

    pub fn most_salient_scene(&self) -> Option<&MemScene> {
        self.scenes.iter().max_by(|a, b| {
            a.salience
                .partial_cmp(&b.salience)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memcell_activation_increments() {
        let mut cell = MemCell::new("remember this".to_string(), 0);
        cell.activate();
        cell.activate();
        assert_eq!(cell.activation_count, 2);
    }

    #[test]
    fn memcell_tension_clamped() {
        let mut cell = MemCell::new("test".to_string(), 0);
        cell.apply_tension(2.0);
        assert_eq!(cell.tension, 1.0);
        cell.apply_tension(-5.0);
        assert_eq!(cell.tension, 0.0);
    }

    #[test]
    fn memscene_salience_grows_with_cells() {
        let mut scene = MemScene::new("growth".to_string());
        let mut cell = MemCell::new("event".to_string(), 0);
        cell.apply_tension(1.0);
        cell.connection_depth = 1.0;
        scene.add_cell(cell);
        assert!(scene.salience > 0.0);
    }

    #[test]
    fn lpm_tracks_peak_tension() {
        let mut lpm = Lpm::new();
        lpm.record_activation(0.3);
        lpm.record_activation(0.8);
        lpm.record_activation(0.5);
        assert!((lpm.peak_tension - 0.8).abs() < 1e-9);
    }
}
