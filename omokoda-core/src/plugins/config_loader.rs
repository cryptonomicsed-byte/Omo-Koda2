use std::path::PathBuf;

use crate::plugins::rule_engine::{Condition, Rule, RuleAction, RuleOperator};

pub struct ConfigLoader {
    pub base_dir: PathBuf,
}

impl ConfigLoader {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    pub fn load_rules(&self) -> Vec<Rule> {
        let pattern = self.base_dir.join("*.local.md");
        let pattern_str = pattern.to_string_lossy();

        glob::glob(&pattern_str)
            .unwrap_or_else(|_| glob::glob("").unwrap())
            .filter_map(|entry| entry.ok())
            .flat_map(|path| {
                std::fs::read_to_string(&path)
                    .map(|content| Self::parse_rules_from_markdown(&content))
                    .unwrap_or_default()
            })
            .collect()
    }

    pub fn parse_rules_from_markdown(content: &str) -> Vec<Rule> {
        let stripped = content.trim_start();
        if !stripped.starts_with("---") {
            return vec![];
        }
        let after_open = &stripped[3..];
        let close = match after_open.find("\n---") {
            Some(i) => i,
            None => return vec![],
        };
        let frontmatter = after_open[..close].trim();

        #[derive(serde::Deserialize)]
        struct RuleEntry {
            name: Option<String>,
            field: Option<String>,
            operator: Option<String>,
            value: Option<String>,
            action: Option<String>,
            message: Option<String>,
        }

        #[derive(serde::Deserialize)]
        struct Header {
            #[serde(default)]
            rules: Vec<RuleEntry>,
        }

        let header: Header = match serde_yaml::from_str(frontmatter) {
            Ok(h) => h,
            Err(_) => return vec![],
        };

        header
            .rules
            .into_iter()
            .filter_map(|entry| {
                let name = entry.name?;
                let field = entry.field?;
                let value = entry.value?;
                let operator = parse_operator(&entry.operator?)?;
                let action = parse_action(&entry.action?, entry.message)?;

                Some(Rule {
                    name,
                    conditions: vec![Condition {
                        field,
                        operator,
                        value,
                    }],
                    action,
                })
            })
            .collect()
    }
}

fn parse_operator(s: &str) -> Option<RuleOperator> {
    match s.to_lowercase().as_str() {
        "contains" => Some(RuleOperator::Contains),
        "not_contains" | "notcontains" => Some(RuleOperator::NotContains),
        "equals" => Some(RuleOperator::Equals),
        "not_equals" | "notequals" => Some(RuleOperator::NotEquals),
        "starts_with" | "startswith" => Some(RuleOperator::StartsWith),
        "ends_with" | "endswith" => Some(RuleOperator::EndsWith),
        "matches" => Some(RuleOperator::Matches),
        _ => None,
    }
}

fn parse_action(action: &str, message: Option<String>) -> Option<RuleAction> {
    match action.to_lowercase().as_str() {
        "block" => Some(RuleAction::Block {
            message: message.unwrap_or_else(|| "blocked by policy".to_string()),
        }),
        "warn" => Some(RuleAction::Warn {
            message: message.unwrap_or_else(|| "policy warning".to_string()),
        }),
        "allow" => Some(RuleAction::Allow),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::rule_engine::{RuleContext, RuleEngine, RuleResult};

    #[test]
    fn parse_valid_rule_from_markdown() {
        let md = r#"---
rules:
  - name: block-sudo
    field: tool_input
    operator: contains
    value: "sudo"
    action: block
    message: "sudo is not allowed"
---
"#;
        let rules = ConfigLoader::parse_rules_from_markdown(md);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name, "block-sudo");

        let engine = RuleEngine::new(rules);
        let mut ctx = RuleContext::new();
        ctx.insert("tool_input", "sudo make install");
        assert!(matches!(engine.evaluate(&ctx), RuleResult::Blocked { .. }));
    }

    #[test]
    fn invalid_operator_ignored() {
        let md = r#"---
rules:
  - name: bad-op
    field: x
    operator: explodes
    value: "v"
    action: block
    message: "nope"
---
"#;
        let rules = ConfigLoader::parse_rules_from_markdown(md);
        assert!(rules.is_empty());
    }

    #[test]
    fn empty_frontmatter_returns_empty() {
        let rules = ConfigLoader::parse_rules_from_markdown("no frontmatter here");
        assert!(rules.is_empty());

        let rules2 = ConfigLoader::parse_rules_from_markdown("---\n---\nbody");
        assert!(rules2.is_empty());
    }

    #[test]
    fn load_rules_from_temp_dir() {
        let dir = tempfile::tempdir().expect("temp dir");
        let file_path = dir.path().join("policy.local.md");
        std::fs::write(
            &file_path,
            "---\nrules:\n  - name: no-curl\n    field: cmd\n    operator: contains\n    value: curl\n    action: warn\n    message: curl flagged\n---\n",
        )
        .unwrap();

        let loader = ConfigLoader::new(dir.path());
        let rules = loader.load_rules();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name, "no-curl");
    }
}
