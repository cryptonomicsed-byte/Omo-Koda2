use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillTier {
    Metadata,
    Core,
    Extended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDef {
    pub name: String,
    pub description: String,
    pub trigger_phrases: Vec<String>,
    pub invocation: Option<String>,
    pub body: String,
    pub tier: u8,
}

impl SkillDef {
    pub fn from_markdown(content: &str) -> Result<Self, String> {
        let (frontmatter, body) = split_frontmatter(content)?;

        #[derive(Deserialize)]
        struct Header {
            name: Option<String>,
            description: Option<String>,
            #[serde(default)]
            trigger: Vec<String>,
            invocation: Option<String>,
            #[serde(default)]
            tier: u8,
        }

        let header: Header =
            serde_yaml::from_str(&frontmatter).map_err(|e| format!("YAML parse error: {e}"))?;

        let name = header.name.ok_or("missing required field: name")?;

        Ok(SkillDef {
            name,
            description: header.description.unwrap_or_default(),
            trigger_phrases: header.trigger,
            invocation: header.invocation,
            body: body.trim_start_matches('\n').to_string(),
            tier: header.tier,
        })
    }

    pub fn matches(&self, input: &str) -> bool {
        if self.trigger_phrases.is_empty() {
            return false;
        }
        let lower = input.to_lowercase();
        self.trigger_phrases
            .iter()
            .any(|t| lower.contains(&t.to_lowercase()))
    }

    pub fn disclosure_tier(&self) -> SkillTier {
        match self.body.len() {
            0..=499 => SkillTier::Metadata,
            500..=1999 => SkillTier::Core,
            _ => SkillTier::Extended,
        }
    }
}

fn split_frontmatter(content: &str) -> Result<(String, String), String> {
    let stripped = content.trim_start();
    if !stripped.starts_with("---") {
        return Ok((String::new(), content.to_string()));
    }
    let after_open = &stripped[3..];
    let close = after_open
        .find("\n---")
        .ok_or("unclosed frontmatter fence")?;
    let frontmatter = after_open[..close].trim().to_string();
    let body = after_open[close + 4..].to_string();
    Ok((frontmatter, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_skill_from_markdown() {
        let md = r#"---
name: odu-lookup
description: Retrieve Odu wisdom for a given cast
trigger:
  - consult odu
  - odu lookup
invocation: /odu-lookup
tier: 2
---
Use this skill to consult the Odu corpus within Omo-Koda2's sovereign memory layer.
"#;
        let skill = SkillDef::from_markdown(md).unwrap();
        assert_eq!(skill.name, "odu-lookup");
        assert_eq!(skill.tier, 2);
        assert_eq!(skill.trigger_phrases, vec!["consult odu", "odu lookup"]);
        assert_eq!(skill.invocation.as_deref(), Some("/odu-lookup"));
        assert!(skill.body.contains("sovereign memory layer"));
    }

    #[test]
    fn matches_returns_true_for_trigger_phrase() {
        let skill = SkillDef {
            name: "synapse-query".to_string(),
            description: String::new(),
            trigger_phrases: vec!["query synapse".to_string(), "check balance".to_string()],
            invocation: None,
            body: String::new(),
            tier: 0,
        };
        assert!(skill.matches("Please query synapse now"));
        assert!(skill.matches("CHECK BALANCE for this agent"));
        assert!(!skill.matches("something unrelated"));
    }
}
