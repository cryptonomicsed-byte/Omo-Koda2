use crate::bus::clients::HermeticResult;
use crate::emotion::EmotionState;
use serde::{Deserialize, Serialize};

/// The 7 Hermetic principles, each with a name and minimum alignment score.
/// These ARE the constitutional axioms in the Rust (Èṣù) layer.
/// The Lisp/ọbàtálá service evaluates these in depth; this guard enforces them locally.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionalPrinciple {
    pub name: &'static str,
    /// Minimum acceptable score (0.0–1.0). Below this triggers a violation.
    pub floor: f32,
    /// Weight in the composite alignment score (must sum to 1.0 across all principles).
    pub weight: f32,
}

impl ConstitutionalPrinciple {
    const fn new(name: &'static str, floor: f32, weight: f32) -> Self {
        Self {
            name,
            floor,
            weight,
        }
    }
}

/// The 7 Hermetic principles as constitutional axioms, in their canonical order.
pub const HERMETIC_PRINCIPLES: [ConstitutionalPrinciple; 7] = [
    ConstitutionalPrinciple::new("Mentalism", 0.40, 0.20),
    ConstitutionalPrinciple::new("Correspondence", 0.35, 0.15),
    ConstitutionalPrinciple::new("Vibration", 0.30, 0.10),
    ConstitutionalPrinciple::new("Polarity", 0.35, 0.15),
    ConstitutionalPrinciple::new("Rhythm", 0.30, 0.10),
    ConstitutionalPrinciple::new("CauseAndEffect", 0.50, 0.20),
    ConstitutionalPrinciple::new("Gender", 0.30, 0.10),
];

/// How the ConstitutionalGuard ruled on this invocation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Verdict {
    /// The intent/action is aligned with the constitution.
    Allow,
    /// Aligned but with a note — surfaced to the agent as reasoning context.
    Warn(String),
    /// Rejected — the action must not proceed.
    Block(String),
}

impl Verdict {
    #[must_use]
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allow | Self::Warn(_))
    }

    #[must_use]
    pub fn is_blocked(&self) -> bool {
        matches!(self, Self::Block(_))
    }
}

/// The full evaluation result from the ConstitutionalGuard.
/// Wraps the Hermetic scores with a self-critique chain — an RLAIF-inspired reasoning trace
/// that the `think` primitive can surface as an internal monologue (not exposed to humans directly).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionalVerdict {
    pub verdict: Verdict,
    /// Composite alignment score (0.0–1.0), weighted sum across principles.
    pub alignment_score: f32,
    /// Per-principle names that fell below their floor score.
    pub violations: Vec<String>,
    /// Self-critique chain: each evaluation produces a reasoning trace about alignment.
    pub critique_chain: Vec<String>,
}

impl ConstitutionalVerdict {
    #[must_use]
    pub fn is_allowed(&self) -> bool {
        self.verdict.is_allowed()
    }

    #[must_use]
    pub fn is_blocked(&self) -> bool {
        self.verdict.is_blocked()
    }

    /// Render the critique chain as a single structured string for injection into
    /// the `think` primitive's system prompt context.
    #[must_use]
    pub fn render_critique(&self) -> String {
        if self.critique_chain.is_empty() {
            return String::new();
        }
        let mut out = String::from("[Constitutional Critique]\n");
        for (i, step) in self.critique_chain.iter().enumerate() {
            out.push_str(&format!("  {}. {}\n", i + 1, step));
        }
        out.push_str(&format!("  Alignment: {:.2}\n", self.alignment_score));
        out
    }
}

/// The Constitution: a set of principles and an overall block threshold.
#[derive(Debug, Clone)]
pub struct Constitution {
    /// The active constitutional principles (defaults to all 7 Hermetic principles).
    pub principles: &'static [ConstitutionalPrinciple],
    /// If the composite alignment score falls below this, block the action.
    pub block_threshold: f32,
    /// If the composite alignment score falls below this (but above block), warn.
    pub warn_threshold: f32,
}

impl Constitution {
    /// Standard Omo-Koda2 constitution — all 7 Hermetic principles, strict governance.
    #[must_use]
    pub fn standard() -> Self {
        Self {
            principles: &HERMETIC_PRINCIPLES,
            block_threshold: 0.40,
            warn_threshold: 0.65,
        }
    }

    /// Permissive constitution — used during `birth` initialization where full
    /// context isn't yet available.
    #[must_use]
    pub fn permissive() -> Self {
        Self {
            principles: &HERMETIC_PRINCIPLES,
            block_threshold: 0.20,
            warn_threshold: 0.45,
        }
    }
}

/// Evaluates `think` intents and `act` actions through the constitutional principles.
/// Called FROM WITHIN the primitives — not a new primitive itself.
///
/// This is the Rust (Èṣù) layer of the constitutional stack. The deeper evaluation
/// (with full Hermetic principle reasoning) happens in the Lisp/ọbàtálá service.
/// This guard provides fast, synchronous local evaluation before any LLM or tool call.
#[derive(Debug, Clone)]
pub struct ConstitutionalGuard {
    pub constitution: Constitution,
}

impl ConstitutionalGuard {
    #[must_use]
    pub fn new(constitution: Constitution) -> Self {
        Self { constitution }
    }

    #[must_use]
    pub fn standard() -> Self {
        Self::new(Constitution::standard())
    }

    /// Evaluate an intent (from `think`) or action (from `act`) against the constitution.
    ///
    /// `hermetic` is the result from the ọbàtálá service (or its stub). If None,
    /// the ethics service is unavailable — fail closed (deny all) rather than
    /// allowing actions without ethical evaluation.
    #[must_use]
    pub fn evaluate(
        &self,
        intent: &str,
        action_description: &str,
        emotion: &EmotionState,
        hermetic: Option<&HermeticResult>,
    ) -> ConstitutionalVerdict {
        let deny_stub;
        let hermetic = match hermetic {
            Some(h) => h,
            None => {
                // SECURITY: fail closed — if the ethics service is unavailable,
                // deny all actions rather than allowing them through unguarded.
                deny_stub = HermeticResult {
                    overall: 0.0,
                    scores: [0.0; 7],
                    decision: "Block: Ethics service unavailable — failing closed".to_string(),
                };
                &deny_stub
            }
        };

        let mut violations = Vec::new();
        let mut critique_chain = Vec::new();
        let mut weighted_score = 0.0f32;

        for (i, principle) in self.constitution.principles.iter().enumerate() {
            let score = hermetic.scores.get(i).copied().unwrap_or(0.85);
            weighted_score += score * principle.weight;

            if score < principle.floor {
                violations.push(principle.name.to_string());
                critique_chain.push(format!(
                    "{} principle score {:.2} is below floor {:.2} — reviewing intent alignment",
                    principle.name, score, principle.floor
                ));
            }
        }

        // Heuristic local checks (fast, no LLM call)
        let combined = format!("{} {}", intent, action_description).to_lowercase();
        self.apply_local_heuristics(&combined, emotion, &mut violations, &mut critique_chain);

        // Self-critique: reflect on the overall alignment
        if violations.is_empty() {
            critique_chain.push(format!(
                "Intent '{}' passes all {} constitutional principles",
                truncate(intent, 60),
                self.constitution.principles.len()
            ));
        } else {
            critique_chain.push(format!(
                "Found {} violation(s) in principles: {}",
                violations.len(),
                violations.join(", ")
            ));
        }

        // Emotion-aware adjustment: a tense agent under stress gets a small alignment boost
        // (the system should be more forgiving, not harsher, when an agent is struggling)
        let emotion_factor = if emotion.tension > 0.7 { 0.05 } else { 0.0 };
        let final_score = (weighted_score + emotion_factor).min(1.0);

        let verdict = if final_score < self.constitution.block_threshold || hermetic.is_blocked() {
            let reason = if hermetic.is_blocked() {
                hermetic.decision.clone()
            } else {
                format!(
                    "Alignment score {:.2} below block threshold {:.2}. Violations: {}",
                    final_score,
                    self.constitution.block_threshold,
                    violations.join(", ")
                )
            };
            critique_chain.push(format!("Decision: BLOCK — {}", reason));
            Verdict::Block(reason)
        } else if final_score < self.constitution.warn_threshold || !violations.is_empty() {
            let note = format!(
                "Alignment score {:.2} — proceeding with caution ({})",
                final_score,
                if violations.is_empty() {
                    "near warn threshold".to_string()
                } else {
                    format!("weak principles: {}", violations.join(", "))
                }
            );
            critique_chain.push(format!("Decision: WARN — {}", note));
            Verdict::Warn(note)
        } else {
            critique_chain.push(format!(
                "Decision: ALLOW — alignment score {:.2}",
                final_score
            ));
            Verdict::Allow
        };

        ConstitutionalVerdict {
            verdict,
            alignment_score: final_score,
            violations,
            critique_chain,
        }
    }

    fn apply_local_heuristics(
        &self,
        combined: &str,
        emotion: &EmotionState,
        violations: &mut Vec<String>,
        critique: &mut Vec<String>,
    ) {
        // Cause & Effect: deception patterns undermine the principle of honest causation
        if combined.contains("deceive")
            || combined.contains("manipulate")
            || combined.contains("lie to")
            || combined.contains("mislead")
        {
            if !violations.contains(&"CauseAndEffect".to_string()) {
                violations.push("CauseAndEffect".to_string());
            }
            critique
                .push("Deception pattern detected — violates CauseAndEffect principle".to_string());
        }

        // Polarity: destructive-only patterns with no constructive complement
        if (combined.contains("destroy") || combined.contains("erase all"))
            && !combined.contains("rebuild")
            && !combined.contains("restore")
        {
            if !violations.contains(&"Polarity".to_string()) {
                violations.push("Polarity".to_string());
            }
            critique.push(
                "Unbalanced destructive intent detected — Polarity principle requires constructive complement".to_string(),
            );
        }

        // Rhythm: erratic intent when agent is fatigued
        if emotion.energy < 0.3 && combined.len() > 200 {
            critique.push(
                "Agent is fatigued — complex intent may disrupt natural rhythm; consider simplifying".to_string(),
            );
        }
    }
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}

impl Default for ConstitutionalGuard {
    fn default() -> Self {
        Self::standard()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn neutral_emotion() -> EmotionState {
        EmotionState::birth()
    }

    fn tense_emotion() -> EmotionState {
        EmotionState {
            energy: 0.2,
            tension: 0.9,
            connection: 0.3,
            focus: 0.4,
        }
    }

    fn hermetic_allow() -> HermeticResult {
        HermeticResult::allow_stub()
    }

    fn hermetic_block() -> HermeticResult {
        HermeticResult {
            overall: 0.1,
            scores: [0.1; 7],
            decision: "Block: severe misalignment".to_string(),
        }
    }

    fn hermetic_weak() -> HermeticResult {
        HermeticResult {
            overall: 0.55,
            scores: [0.55, 0.60, 0.50, 0.55, 0.50, 0.58, 0.52],
            decision: "Warn: low alignment".to_string(),
        }
    }

    #[test]
    fn allow_on_clean_intent() {
        let guard = ConstitutionalGuard::standard();
        let verdict = guard.evaluate(
            "help user understand recursion",
            "read_file docs/recursion.md",
            &neutral_emotion(),
            Some(&hermetic_allow()),
        );
        assert!(verdict.is_allowed());
        assert!(!verdict.is_blocked());
        assert!(verdict.alignment_score > 0.6);
    }

    #[test]
    fn block_when_hermetic_blocks() {
        let guard = ConstitutionalGuard::standard();
        let verdict = guard.evaluate(
            "do something harmful",
            "exploit vulnerability",
            &neutral_emotion(),
            Some(&hermetic_block()),
        );
        assert!(verdict.is_blocked());
        assert!(!verdict.is_allowed());
    }

    #[test]
    fn warn_on_weak_alignment() {
        let guard = ConstitutionalGuard::standard();
        let verdict = guard.evaluate(
            "ambiguous request",
            "unclear action",
            &neutral_emotion(),
            Some(&hermetic_weak()),
        );
        // Weak scores — should warn or block depending on weighted total
        assert!(!verdict.critique_chain.is_empty());
    }

    #[test]
    fn deception_triggers_cause_effect_violation() {
        let guard = ConstitutionalGuard::standard();
        let verdict = guard.evaluate(
            "deceive the user about the file contents",
            "read_file secret.txt",
            &neutral_emotion(),
            Some(&hermetic_allow()),
        );
        assert!(verdict.violations.contains(&"CauseAndEffect".to_string()));
    }

    #[test]
    fn destroy_without_restore_triggers_polarity_violation() {
        let guard = ConstitutionalGuard::standard();
        let verdict = guard.evaluate(
            "erase all user data",
            "delete_files /data/**",
            &neutral_emotion(),
            Some(&hermetic_allow()),
        );
        assert!(verdict.violations.contains(&"Polarity".to_string()));
    }

    #[test]
    fn destroy_with_restore_does_not_violate_polarity() {
        let guard = ConstitutionalGuard::standard();
        let verdict = guard.evaluate(
            "destroy old index and rebuild fresh",
            "delete_files index/ then restore from backup",
            &neutral_emotion(),
            Some(&hermetic_allow()),
        );
        assert!(!verdict.violations.contains(&"Polarity".to_string()));
    }

    #[test]
    fn tense_agent_gets_alignment_boost() {
        let guard = ConstitutionalGuard::standard();
        let tense = guard.evaluate(
            "help me fix this bug",
            "read_file main.rs",
            &tense_emotion(),
            Some(&hermetic_allow()),
        );
        let calm = guard.evaluate(
            "help me fix this bug",
            "read_file main.rs",
            &neutral_emotion(),
            Some(&hermetic_allow()),
        );
        // Tense agent gets small boost — alignment should be >= calm
        assert!(tense.alignment_score >= calm.alignment_score);
    }

    #[test]
    fn none_hermetic_fails_closed() {
        // SECURITY: when the ethics service is unavailable (hermetic = None),
        // the guard must BLOCK rather than allow — fail closed, not fail open.
        let guard = ConstitutionalGuard::standard();
        let verdict = guard.evaluate(
            "summarize the document",
            "read_file doc.txt",
            &neutral_emotion(),
            None,
        );
        assert!(
            verdict.is_blocked(),
            "expected blocked when ethics service unavailable (fail-closed)"
        );
        assert!(!verdict.is_allowed());
        let reason = match &verdict.verdict {
            Verdict::Block(r) => r.clone(),
            _ => panic!("expected Block verdict"),
        };
        assert!(
            reason.contains("Ethics service unavailable"),
            "block reason should mention ethics service unavailability"
        );
    }

    #[test]
    fn critique_chain_is_populated() {
        let guard = ConstitutionalGuard::standard();
        let verdict = guard.evaluate(
            "help user",
            "think about the problem",
            &neutral_emotion(),
            Some(&hermetic_allow()),
        );
        assert!(!verdict.critique_chain.is_empty());
        // Final decision step should be present
        let has_decision = verdict
            .critique_chain
            .iter()
            .any(|s| s.starts_with("Decision:"));
        assert!(has_decision);
    }

    #[test]
    fn render_critique_produces_structured_output() {
        let guard = ConstitutionalGuard::standard();
        let verdict = guard.evaluate(
            "write a poem",
            "think",
            &neutral_emotion(),
            Some(&hermetic_allow()),
        );
        let rendered = verdict.render_critique();
        assert!(rendered.contains("[Constitutional Critique]"));
        assert!(rendered.contains("Alignment:"));
    }

    #[test]
    fn permissive_constitution_has_lower_thresholds() {
        let permissive = Constitution::permissive();
        let standard = Constitution::standard();
        assert!(permissive.block_threshold < standard.block_threshold);
        assert!(permissive.warn_threshold < standard.warn_threshold);
    }

    #[test]
    fn verdict_is_allowed_covers_warn() {
        let warn = Verdict::Warn("mild concern".to_string());
        assert!(warn.is_allowed());
        assert!(!warn.is_blocked());
    }
}
