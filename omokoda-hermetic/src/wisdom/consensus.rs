use serde::{Deserialize, Serialize};

/// A model with its historical reliability weight.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelWeight {
    pub name: String,
    pub weight: f64,
}

impl ModelWeight {
    pub fn new(name: impl Into<String>, weight: f64) -> Self {
        Self {
            name: name.into(),
            weight: weight.clamp(0.0, 1.0),
        }
    }
}

/// A model's opinion on a decision.
#[derive(Debug, Clone)]
pub struct ModelOpinion {
    pub model: ModelWeight,
    pub score: f64, // 0.0 = reject, 1.0 = approve
    pub rationale: String,
}

/// Weighted consensus result.
#[derive(Debug, Clone)]
pub struct WeightedResult {
    pub weighted_score: f64,
    pub total_weight: f64,
    pub model_count: usize,
}

impl WeightedResult {
    pub fn approved(&self) -> bool {
        self.weighted_score / self.total_weight > 0.5
    }
}

/// Consensus engine — weighted average over multiple model opinions.
pub struct ConsensusEngine;

impl ConsensusEngine {
    pub fn aggregate(opinions: &[ModelOpinion]) -> WeightedResult {
        let total_weight: f64 = opinions.iter().map(|o| o.model.weight).sum();
        let weighted_score: f64 = opinions.iter().map(|o| o.score * o.model.weight).sum();
        WeightedResult {
            weighted_score,
            total_weight,
            model_count: opinions.len(),
        }
    }
}

/// Standard model weight table from Twelve-thrones.
pub fn default_model_weights() -> Vec<ModelWeight> {
    vec![
        ModelWeight::new("claude", 0.98),
        ModelWeight::new("gpt4o", 0.96),
        ModelWeight::new("gemini", 0.93),
        ModelWeight::new("llama", 0.88),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unanimous_approval() {
        let opinions = vec![
            ModelOpinion {
                model: ModelWeight::new("a", 0.9),
                score: 1.0,
                rationale: "approve".to_string(),
            },
            ModelOpinion {
                model: ModelWeight::new("b", 0.8),
                score: 1.0,
                rationale: "approve".to_string(),
            },
        ];
        let result = ConsensusEngine::aggregate(&opinions);
        assert!(result.approved());
    }

    #[test]
    fn unanimous_rejection() {
        let opinions = vec![ModelOpinion {
            model: ModelWeight::new("a", 0.9),
            score: 0.0,
            rationale: "reject".to_string(),
        }];
        let result = ConsensusEngine::aggregate(&opinions);
        assert!(!result.approved());
    }

    #[test]
    fn default_weights_include_claude() {
        let weights = default_model_weights();
        assert!(weights.iter().any(|m| m.name == "claude"));
        let claude = weights.iter().find(|m| m.name == "claude").unwrap();
        assert!((claude.weight - 0.98).abs() < 1e-9);
    }
}
