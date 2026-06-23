use serde::{Deserialize, Serialize};

use crate::usage::TokenUsage;

/// Per-query token budget — mirrors QueryEngine's token budgeting logic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudget {
    pub input_limit: u64,
    pub output_limit: u64,
    pub used_input: u64,
    pub used_output: u64,
}

impl Default for TokenBudget {
    fn default() -> Self {
        Self {
            input_limit: 200_000,
            output_limit: 32_000,
            used_input: 0,
            used_output: 0,
        }
    }
}

impl TokenBudget {
    pub fn new(input_limit: u64, output_limit: u64) -> Self {
        Self {
            input_limit,
            output_limit,
            used_input: 0,
            used_output: 0,
        }
    }

    pub fn record(&mut self, usage: &TokenUsage) {
        self.used_input += usage.input_tokens as u64;
        self.used_output += usage.output_tokens as u64;
    }

    pub fn input_remaining(&self) -> u64 {
        self.input_limit.saturating_sub(self.used_input)
    }

    pub fn output_remaining(&self) -> u64 {
        self.output_limit.saturating_sub(self.used_output)
    }

    pub fn is_exceeded(&self) -> bool {
        self.used_input >= self.input_limit || self.used_output >= self.output_limit
    }

    pub fn utilization(&self) -> f32 {
        let in_pct = self.used_input as f32 / self.input_limit.max(1) as f32;
        let out_pct = self.used_output as f32 / self.output_limit.max(1) as f32;
        in_pct.max(out_pct)
    }
}

/// Conditions that halt a query run
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StopCondition {
    /// Token budget fully consumed
    BudgetExceeded,
    /// Reached max allowed turns
    MaxTurnsReached,
    /// Explicit stop requested (e.g. /compact or user interrupt)
    ExplicitStop,
    /// Agent synapse (energy) too low
    SynapseExhausted,
    /// A hook returned Deny
    HookDenied(String),
    /// Provider returned an unrecoverable error
    ProviderError(String),
}

/// A registered stop hook — called before each query turn
pub struct StopHook {
    pub name: String,
    #[allow(clippy::type_complexity)]
    pub check: Box<dyn Fn(&QueryState) -> Option<StopCondition> + Send + Sync>,
}

impl StopHook {
    pub fn new(
        name: impl Into<String>,
        check: impl Fn(&QueryState) -> Option<StopCondition> + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            check: Box::new(check),
        }
    }

    /// Built-in: halt when budget is exceeded
    pub fn budget_guard() -> Self {
        Self::new("budget_guard", |state| {
            if state.budget.is_exceeded() {
                Some(StopCondition::BudgetExceeded)
            } else {
                None
            }
        })
    }

    /// Built-in: halt when turn count reaches max
    pub fn max_turns_guard(max: u32) -> Self {
        Self::new("max_turns_guard", move |state| {
            if state.turn_count >= max {
                Some(StopCondition::MaxTurnsReached)
            } else {
                None
            }
        })
    }

    /// Built-in: halt when synapse falls below floor
    pub fn synapse_guard(floor: f64) -> Self {
        Self::new("synapse_guard", move |state| {
            if state.synapse_remaining < floor {
                Some(StopCondition::SynapseExhausted)
            } else {
                None
            }
        })
    }
}

/// Dependency between steps within a query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryDependency {
    pub from_step: String,
    pub to_step: String,
    pub kind: DependencyKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyKind {
    /// `to` must complete before `from` starts
    Sequential,
    /// `from` uses output of `to`
    DataFlow,
    /// `from` is only needed if `to` fails
    Fallback,
}

/// Configuration for a query session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryConfig {
    pub max_turns: u32,
    pub budget: TokenBudget,
    pub synapse_floor: f64,
    pub allow_tool_use: bool,
    pub allow_subagents: bool,
    pub private: bool,
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            max_turns: 25,
            budget: TokenBudget::default(),
            synapse_floor: 500.0,
            allow_tool_use: true,
            allow_subagents: false,
            private: false,
        }
    }
}

/// Live state of a running query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryState {
    pub turn_count: u32,
    pub budget: TokenBudget,
    pub synapse_remaining: f64,
    pub completed_steps: Vec<String>,
    pub stop_reason: Option<StopCondition>,
}

impl QueryState {
    pub fn new(config: &QueryConfig, initial_synapse: f64) -> Self {
        Self {
            turn_count: 0,
            budget: config.budget.clone(),
            synapse_remaining: initial_synapse,
            completed_steps: Vec::new(),
            stop_reason: None,
        }
    }

    pub fn is_stopped(&self) -> bool {
        self.stop_reason.is_some()
    }
}

/// Central query orchestrator — runs stop-hook checks each turn, tracks budget/deps.
pub struct QueryEngine {
    config: QueryConfig,
    stop_hooks: Vec<StopHook>,
    dependencies: Vec<QueryDependency>,
}

impl QueryEngine {
    pub fn new(config: QueryConfig) -> Self {
        let mut engine = Self {
            config,
            stop_hooks: Vec::new(),
            dependencies: Vec::new(),
        };
        // Install built-in guards
        let max = engine.config.max_turns;
        let floor = engine.config.synapse_floor;
        engine.stop_hooks.push(StopHook::budget_guard());
        engine.stop_hooks.push(StopHook::max_turns_guard(max));
        engine.stop_hooks.push(StopHook::synapse_guard(floor));
        engine
    }

    pub fn with_defaults() -> Self {
        Self::new(QueryConfig::default())
    }

    pub fn add_stop_hook(&mut self, hook: StopHook) {
        self.stop_hooks.push(hook);
    }

    pub fn add_dependency(&mut self, dep: QueryDependency) {
        self.dependencies.push(dep);
    }

    pub fn config(&self) -> &QueryConfig {
        &self.config
    }

    /// Check all stop hooks; returns the first triggered condition, if any.
    pub fn check_stop(&self, state: &QueryState) -> Option<StopCondition> {
        for hook in &self.stop_hooks {
            if let Some(cond) = (hook.check)(state) {
                return Some(cond);
            }
        }
        None
    }

    /// Advance state by one turn, recording usage and synapse cost.
    pub fn advance_turn(&self, state: &mut QueryState, usage: &TokenUsage, synapse_cost: f64) {
        state.turn_count += 1;
        state.budget.record(usage);
        state.synapse_remaining = (state.synapse_remaining - synapse_cost).max(0.0);
        if let Some(cond) = self.check_stop(state) {
            state.stop_reason = Some(cond);
        }
    }

    /// Returns steps that are ready to execute (all their Sequential deps are done).
    pub fn ready_steps<'a>(&self, state: &QueryState, all_steps: &[&'a str]) -> Vec<&'a str> {
        all_steps
            .iter()
            .copied()
            .filter(|&step| {
                if state.completed_steps.contains(&step.to_string()) {
                    return false;
                }
                // All sequential deps must be completed
                self.dependencies
                    .iter()
                    .filter(|d| d.from_step == step && d.kind == DependencyKind::Sequential)
                    .all(|d| state.completed_steps.contains(&d.to_step))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_budget_tracking() {
        let mut budget = TokenBudget::new(1000, 500);
        assert!(!budget.is_exceeded());
        budget.record(&TokenUsage {
            input_tokens: 800,
            output_tokens: 200,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
        });
        assert!(!budget.is_exceeded());
        budget.record(&TokenUsage {
            input_tokens: 300,
            output_tokens: 0,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
        });
        assert!(budget.is_exceeded());
    }

    #[test]
    fn test_stop_hooks_fire() {
        let config = QueryConfig {
            max_turns: 3,
            ..Default::default()
        };
        let engine = QueryEngine::new(config.clone());
        let mut state = QueryState::new(&config, 10_000.0);

        state.turn_count = 3;
        let cond = engine.check_stop(&state);
        assert_eq!(cond, Some(StopCondition::MaxTurnsReached));
    }

    #[test]
    fn test_ready_steps_with_deps() {
        let mut engine = QueryEngine::with_defaults();
        engine.add_dependency(QueryDependency {
            from_step: "b".to_string(),
            to_step: "a".to_string(),
            kind: DependencyKind::Sequential,
        });
        let config = QueryConfig::default();
        let state = QueryState::new(&config, 10_000.0);
        let ready = engine.ready_steps(&state, &["a", "b"]);
        assert_eq!(ready, vec!["a"]);
    }

    #[test]
    fn test_budget_utilization() {
        let mut budget = TokenBudget::new(100, 50);
        budget.record(&TokenUsage {
            input_tokens: 50,
            output_tokens: 0,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
        });
        let util = budget.utilization();
        assert!((util - 0.5).abs() < 0.01);
    }
}
