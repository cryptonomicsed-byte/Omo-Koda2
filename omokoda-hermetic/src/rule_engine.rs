//! Hermetic Rule Engine — Pattern 70.
//!
//! Pattern-matches agent state against a set of rules to decide whether
//! a hook should fire, be skipped, or produce a modified outcome.
//!
//! Ports Claw's `hookify/core/rule_engine.py` to Rust with type-safe conditions.

use serde::{Deserialize, Serialize};

// ── Conditions ────────────────────────────────────────────────────────────────

/// A single condition that must match for a rule to fire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    /// Hook event name equals value (e.g. "pre_think")
    EventIs(String),
    /// Tool name equals value
    ToolIs(String),
    /// Tool name matches a glob pattern (e.g. "bash*")
    ToolMatches(String),
    /// Agent tier is at least this level
    TierAtLeast(u8),
    /// Agent tier is at most this level
    TierAtMost(u8),
    /// A named flag in the agent context is truthy
    FlagSet(String),
    /// A named flag is absent or false
    FlagUnset(String),
    /// Logical NOT of an inner condition
    Not(Box<RuleCondition>),
    /// All inner conditions must hold
    All(Vec<RuleCondition>),
    /// At least one inner condition must hold
    Any(Vec<RuleCondition>),
}

impl RuleCondition {
    pub fn matches(&self, ctx: &EvalContext) -> bool {
        match self {
            Self::EventIs(e) => ctx.event.as_deref() == Some(e.as_str()),
            Self::ToolIs(t) => ctx.tool.as_deref() == Some(t.as_str()),
            Self::ToolMatches(pattern) => ctx
                .tool
                .as_deref()
                .map(|t| glob_match(pattern, t))
                .unwrap_or(false),
            Self::TierAtLeast(min) => ctx.tier >= *min,
            Self::TierAtMost(max) => ctx.tier <= *max,
            Self::FlagSet(flag) => ctx.flags.contains(flag.as_str()),
            Self::FlagUnset(flag) => !ctx.flags.contains(flag.as_str()),
            Self::Not(inner) => !inner.matches(ctx),
            Self::All(conds) => conds.iter().all(|c| c.matches(ctx)),
            Self::Any(conds) => conds.iter().any(|c| c.matches(ctx)),
        }
    }
}

// ── Actions ───────────────────────────────────────────────────────────────────

/// What the rule does when its conditions match.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RuleAction {
    /// Let the hook fire normally
    Allow,
    /// Block the hook / operation with a reason
    Block(String),
    /// Emit a warning but continue
    Warn(String),
    /// Skip this hook silently (passthrough)
    Skip,
    /// Delegate to another agent
    Delegate { agent_id: String },
}

// ── Rule ──────────────────────────────────────────────────────────────────────

/// A single named rule: if all conditions match, apply the action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookRule {
    pub id: String,
    pub description: String,
    /// Lower priority numbers evaluated first
    pub priority: i32,
    pub condition: RuleCondition,
    pub action: RuleAction,
    /// If true, stop evaluating further rules once this one fires
    pub terminal: bool,
}

impl HookRule {
    pub fn new(id: impl Into<String>, condition: RuleCondition, action: RuleAction) -> Self {
        Self {
            id: id.into(),
            description: String::new(),
            priority: 0,
            condition,
            action,
            terminal: false,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_priority(mut self, p: i32) -> Self {
        self.priority = p;
        self
    }

    pub fn terminal(mut self) -> Self {
        self.terminal = true;
        self
    }

    pub fn matches(&self, ctx: &EvalContext) -> bool {
        self.condition.matches(ctx)
    }
}

// ── Context ───────────────────────────────────────────────────────────────────

/// Snapshot of agent state passed to the rule engine for evaluation.
#[derive(Debug, Clone, Default)]
pub struct EvalContext {
    /// Current hook event name (e.g. "pre_think")
    pub event: Option<String>,
    /// Tool being invoked, if any
    pub tool: Option<String>,
    /// Agent capability tier
    pub tier: u8,
    /// Set of active feature flags / agent state flags
    pub flags: std::collections::HashSet<String>,
    /// Additional string key-value metadata
    pub meta: std::collections::HashMap<String, String>,
}

impl EvalContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_event(mut self, event: impl Into<String>) -> Self {
        self.event = Some(event.into());
        self
    }

    pub fn with_tool(mut self, tool: impl Into<String>) -> Self {
        self.tool = Some(tool.into());
        self
    }

    pub fn with_tier(mut self, tier: u8) -> Self {
        self.tier = tier;
        self
    }

    pub fn with_flag(mut self, flag: impl Into<String>) -> Self {
        self.flags.insert(flag.into());
        self
    }
}

// ── Engine ────────────────────────────────────────────────────────────────────

/// Evaluates an ordered set of `HookRule`s against an `EvalContext`.
pub struct RuleEngine {
    rules: Vec<HookRule>,
}

impl RuleEngine {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, rule: HookRule) {
        self.rules.push(rule);
        self.rules.sort_by_key(|r| r.priority);
    }

    /// Evaluate all rules; return the action of the first matching rule.
    /// If no rule matches, returns `RuleAction::Allow`.
    pub fn evaluate(&self, ctx: &EvalContext) -> RuleAction {
        for rule in &self.rules {
            if rule.matches(ctx) {
                if rule.terminal {
                    return rule.action.clone();
                }
                // Non-terminal: only return if it's not Allow (blocking/warn take precedence)
                if rule.action != RuleAction::Allow && rule.action != RuleAction::Skip {
                    return rule.action.clone();
                }
            }
        }
        RuleAction::Allow
    }

    /// Return all matching rules (for inspection/debugging).
    pub fn matching_rules<'a>(&'a self, ctx: &EvalContext) -> Vec<&'a HookRule> {
        self.rules.iter().filter(|r| r.matches(ctx)).collect()
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ── Glob matching ─────────────────────────────────────────────────────────────

fn glob_match(pattern: &str, value: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return value.starts_with(prefix);
    }
    if let Some(suffix) = pattern.strip_prefix('*') {
        return value.ends_with(suffix);
    }
    pattern == value
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> EvalContext {
        EvalContext::new()
            .with_event("pre_act")
            .with_tool("bash")
            .with_tier(1)
    }

    #[test]
    fn test_event_is_matches() {
        let c = RuleCondition::EventIs("pre_act".to_string());
        assert!(c.matches(&ctx()));
        let c2 = RuleCondition::EventIs("post_act".to_string());
        assert!(!c2.matches(&ctx()));
    }

    #[test]
    fn test_tool_is_matches() {
        let c = RuleCondition::ToolIs("bash".to_string());
        assert!(c.matches(&ctx()));
    }

    #[test]
    fn test_tool_matches_glob() {
        let c = RuleCondition::ToolMatches("ba*".to_string());
        assert!(c.matches(&ctx()));
        let c2 = RuleCondition::ToolMatches("*sh".to_string());
        assert!(c2.matches(&ctx()));
        let c3 = RuleCondition::ToolMatches("*".to_string());
        assert!(c3.matches(&ctx()));
    }

    #[test]
    fn test_tier_at_least() {
        let c = RuleCondition::TierAtLeast(1);
        assert!(c.matches(&ctx()));
        let c2 = RuleCondition::TierAtLeast(2);
        assert!(!c2.matches(&ctx()));
    }

    #[test]
    fn test_not_condition() {
        let c = RuleCondition::Not(Box::new(RuleCondition::ToolIs("python".to_string())));
        assert!(c.matches(&ctx())); // tool is bash, not python
    }

    #[test]
    fn test_all_condition() {
        let c = RuleCondition::All(vec![
            RuleCondition::EventIs("pre_act".to_string()),
            RuleCondition::ToolIs("bash".to_string()),
        ]);
        assert!(c.matches(&ctx()));
    }

    #[test]
    fn test_any_condition() {
        let c = RuleCondition::Any(vec![
            RuleCondition::EventIs("post_act".to_string()),
            RuleCondition::ToolIs("bash".to_string()),
        ]);
        assert!(c.matches(&ctx()));
    }

    #[test]
    fn test_flag_set_unset() {
        let ctx_with_flag = ctx().with_flag("sandbox");
        assert!(RuleCondition::FlagSet("sandbox".to_string()).matches(&ctx_with_flag));
        assert!(!RuleCondition::FlagUnset("sandbox".to_string()).matches(&ctx_with_flag));
        assert!(!RuleCondition::FlagSet("sandbox".to_string()).matches(&ctx()));
    }

    #[test]
    fn test_engine_returns_allow_when_no_rules() {
        let engine = RuleEngine::new();
        assert_eq!(engine.evaluate(&ctx()), RuleAction::Allow);
    }

    #[test]
    fn test_engine_applies_first_matching_blocking_rule() {
        let mut engine = RuleEngine::new();
        engine.add_rule(
            HookRule::new(
                "block-bash",
                RuleCondition::ToolIs("bash".to_string()),
                RuleAction::Block("bash not allowed".to_string()),
            )
            .with_priority(1),
        );
        engine.add_rule(
            HookRule::new(
                "allow-all",
                RuleCondition::EventIs("pre_act".to_string()),
                RuleAction::Allow,
            )
            .with_priority(10),
        );
        assert!(matches!(engine.evaluate(&ctx()), RuleAction::Block(_)));
    }

    #[test]
    fn test_engine_terminal_rule_stops_evaluation() {
        let mut engine = RuleEngine::new();
        engine.add_rule(
            HookRule::new(
                "allow-terminal",
                RuleCondition::EventIs("pre_act".to_string()),
                RuleAction::Allow,
            )
            .terminal()
            .with_priority(1),
        );
        engine.add_rule(
            HookRule::new(
                "block-later",
                RuleCondition::ToolIs("bash".to_string()),
                RuleAction::Block("should not reach".to_string()),
            )
            .with_priority(10),
        );
        // Terminal Allow rule fires first, stops evaluation
        assert_eq!(engine.evaluate(&ctx()), RuleAction::Allow);
    }

    #[test]
    fn test_matching_rules_inspection() {
        let mut engine = RuleEngine::new();
        engine.add_rule(HookRule::new(
            "r1",
            RuleCondition::EventIs("pre_act".to_string()),
            RuleAction::Warn("heads up".to_string()),
        ));
        engine.add_rule(HookRule::new(
            "r2",
            RuleCondition::EventIs("post_act".to_string()),
            RuleAction::Allow,
        ));
        let matching = engine.matching_rules(&ctx());
        assert_eq!(matching.len(), 1);
        assert_eq!(matching[0].id, "r1");
    }
}
