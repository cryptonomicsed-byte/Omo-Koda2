use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum RuleOperator {
    Contains,
    NotContains,
    Equals,
    NotEquals,
    StartsWith,
    EndsWith,
    Matches,
}

#[derive(Debug, Clone)]
pub struct Condition {
    pub field: String,
    pub operator: RuleOperator,
    pub value: String,
}

#[derive(Debug, Clone)]
pub enum RuleAction {
    Block { message: String },
    Warn { message: String },
    Allow,
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub name: String,
    pub conditions: Vec<Condition>,
    pub action: RuleAction,
}

#[derive(Debug, Clone, Default)]
pub struct RuleContext {
    pub fields: HashMap<String, String>,
}

impl RuleContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.fields.insert(key.into(), value.into());
    }
}

#[derive(Debug, Clone)]
pub enum RuleResult {
    Blocked { rule: String, message: String },
    Warned { rule: String, message: String },
    Allowed,
}

pub struct RuleEngine {
    rules: Vec<Rule>,
}

impl RuleEngine {
    pub fn new(rules: Vec<Rule>) -> Self {
        Self { rules }
    }

    pub fn evaluate(&self, ctx: &RuleContext) -> RuleResult {
        let mut first_warn: Option<RuleResult> = None;

        for rule in &self.rules {
            if !all_conditions_match(&rule.conditions, ctx) {
                continue;
            }
            match &rule.action {
                RuleAction::Block { message } => {
                    return RuleResult::Blocked {
                        rule: rule.name.clone(),
                        message: message.clone(),
                    };
                }
                RuleAction::Warn { message } => {
                    if first_warn.is_none() {
                        first_warn = Some(RuleResult::Warned {
                            rule: rule.name.clone(),
                            message: message.clone(),
                        });
                    }
                }
                RuleAction::Allow => {}
            }
        }

        first_warn.unwrap_or(RuleResult::Allowed)
    }
}

fn all_conditions_match(conditions: &[Condition], ctx: &RuleContext) -> bool {
    conditions.iter().all(|c| condition_matches(c, ctx))
}

fn condition_matches(cond: &Condition, ctx: &RuleContext) -> bool {
    let field_val = match ctx.fields.get(&cond.field) {
        Some(v) => v.as_str(),
        None => return false,
    };

    match &cond.operator {
        RuleOperator::Contains => field_val.contains(cond.value.as_str()),
        RuleOperator::NotContains => !field_val.contains(cond.value.as_str()),
        RuleOperator::Equals => field_val == cond.value,
        RuleOperator::NotEquals => field_val != cond.value,
        RuleOperator::StartsWith => field_val.starts_with(cond.value.as_str()),
        RuleOperator::EndsWith => field_val.ends_with(cond.value.as_str()),
        RuleOperator::Matches => Regex::new(&cond.value)
            .map(|re| re.is_match(field_val))
            .unwrap_or(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block_rule(field: &str, op: RuleOperator, val: &str) -> Rule {
        Rule {
            name: "test-block".to_string(),
            conditions: vec![Condition {
                field: field.to_string(),
                operator: op,
                value: val.to_string(),
            }],
            action: RuleAction::Block {
                message: "blocked".to_string(),
            },
        }
    }

    fn warn_rule(field: &str, op: RuleOperator, val: &str) -> Rule {
        Rule {
            name: "test-warn".to_string(),
            conditions: vec![Condition {
                field: field.to_string(),
                operator: op,
                value: val.to_string(),
            }],
            action: RuleAction::Warn {
                message: "warned".to_string(),
            },
        }
    }

    #[test]
    fn block_rule_triggers() {
        let engine = RuleEngine::new(vec![block_rule(
            "tool_input",
            RuleOperator::Contains,
            "sudo",
        )]);
        let mut ctx = RuleContext::new();
        ctx.insert("tool_input", "sudo rm -rf /");
        match engine.evaluate(&ctx) {
            RuleResult::Blocked { rule, .. } => assert_eq!(rule, "test-block"),
            other => panic!("expected Blocked, got {other:?}"),
        }
    }

    #[test]
    fn warn_rule_triggers() {
        let engine = RuleEngine::new(vec![warn_rule(
            "tool_input",
            RuleOperator::StartsWith,
            "rm",
        )]);
        let mut ctx = RuleContext::new();
        ctx.insert("tool_input", "rm file.txt");
        match engine.evaluate(&ctx) {
            RuleResult::Warned { rule, .. } => assert_eq!(rule, "test-warn"),
            other => panic!("expected Warned, got {other:?}"),
        }
    }

    #[test]
    fn allow_when_no_match() {
        let engine = RuleEngine::new(vec![block_rule(
            "tool_input",
            RuleOperator::Contains,
            "sudo",
        )]);
        let mut ctx = RuleContext::new();
        ctx.insert("tool_input", "ls -la");
        assert!(matches!(engine.evaluate(&ctx), RuleResult::Allowed));
    }

    #[test]
    fn multiple_conditions_and_logic() {
        let rule = Rule {
            name: "and-rule".to_string(),
            conditions: vec![
                Condition {
                    field: "tool".to_string(),
                    operator: RuleOperator::Equals,
                    value: "bash".to_string(),
                },
                Condition {
                    field: "input".to_string(),
                    operator: RuleOperator::Contains,
                    value: "curl".to_string(),
                },
            ],
            action: RuleAction::Block {
                message: "no curl in bash".to_string(),
            },
        };
        let engine = RuleEngine::new(vec![rule]);

        let mut ctx_both = RuleContext::new();
        ctx_both.insert("tool", "bash");
        ctx_both.insert("input", "curl http://example.com");
        assert!(matches!(
            engine.evaluate(&ctx_both),
            RuleResult::Blocked { .. }
        ));

        let mut ctx_one = RuleContext::new();
        ctx_one.insert("tool", "bash");
        ctx_one.insert("input", "echo hello");
        assert!(matches!(engine.evaluate(&ctx_one), RuleResult::Allowed));
    }

    #[test]
    fn regex_operator() {
        let engine = RuleEngine::new(vec![block_rule(
            "command",
            RuleOperator::Matches,
            r"^\s*sudo\b",
        )]);
        let mut ctx = RuleContext::new();
        ctx.insert("command", "  sudo systemctl restart");
        assert!(matches!(engine.evaluate(&ctx), RuleResult::Blocked { .. }));

        let mut ctx2 = RuleContext::new();
        ctx2.insert("command", "echo sudo is not used");
        assert!(matches!(engine.evaluate(&ctx2), RuleResult::Allowed));
    }
}
