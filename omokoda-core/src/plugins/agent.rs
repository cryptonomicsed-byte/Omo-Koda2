use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDef {
    pub name: String,
    pub description: String,
    pub model: Option<String>,
    pub color: Option<String>,
    pub tools: Vec<String>,
    pub system_prompt: String,
    pub trigger_phrases: Vec<String>,
}

impl AgentDef {
    pub fn from_markdown(content: &str) -> Result<Self, String> {
        let (frontmatter, body) = split_frontmatter(content)?;

        #[derive(Deserialize)]
        struct Header {
            name: Option<String>,
            description: Option<String>,
            model: Option<String>,
            color: Option<String>,
            tools: Option<Vec<String>>,
            #[serde(default)]
            trigger: Vec<String>,
        }

        let header: Header =
            serde_yaml::from_str(&frontmatter).map_err(|e| format!("YAML parse error: {e}"))?;

        let name = header.name.ok_or("missing required field: name")?;

        Ok(AgentDef {
            name,
            description: header.description.unwrap_or_default(),
            model: header.model,
            color: header.color,
            tools: header.tools.unwrap_or_default(),
            system_prompt: body.trim_start_matches('\n').to_string(),
            trigger_phrases: header.trigger,
        })
    }

    pub fn matches_trigger(&self, input: &str) -> bool {
        if self.trigger_phrases.is_empty() {
            return false;
        }
        let lower = input.to_lowercase();
        self.trigger_phrases
            .iter()
            .any(|t| lower.contains(&t.to_lowercase()))
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
    fn parse_valid_agent() {
        let md = "---\nname: researcher\ndescription: Deep research subagent\nmodel: claude-sonnet-4-6\ncolor: \"#4B9CD3\"\ntools:\n  - web_search\n  - read_file\ntrigger:\n  - research this\n  - look up\n---\nYou are a sovereign research agent operating within Omo-Koda2.\n";
        let agent = AgentDef::from_markdown(md).unwrap();
        assert_eq!(agent.name, "researcher");
        assert_eq!(agent.tools, vec!["web_search", "read_file"]);
        assert_eq!(agent.trigger_phrases, vec!["research this", "look up"]);
        assert!(agent.system_prompt.contains("sovereign research agent"));
    }

    #[test]
    fn trigger_matching_works() {
        let agent = AgentDef {
            name: "scout".to_string(),
            description: String::new(),
            model: None,
            color: None,
            tools: vec![],
            system_prompt: String::new(),
            trigger_phrases: vec!["scan this".to_string(), "discover".to_string()],
        };
        assert!(agent.matches_trigger("Please SCAN THIS directory"));
        assert!(agent.matches_trigger("Discover all files"));
        assert!(!agent.matches_trigger("do something else"));
    }

    #[test]
    fn empty_triggers_never_match() {
        let agent = AgentDef {
            name: "idle".to_string(),
            description: String::new(),
            model: None,
            color: None,
            tools: vec![],
            system_prompt: String::new(),
            trigger_phrases: vec![],
        };
        assert!(!agent.matches_trigger("anything at all"));
        assert!(!agent.matches_trigger(""));
    }
}
