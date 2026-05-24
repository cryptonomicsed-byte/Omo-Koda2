//! Declarative agent definitions: YAML frontmatter + system prompt body.
//!
//! Agents are defined as markdown files with YAML frontmatter.
//!
//! Frontmatter schema:
//! ```yaml
//! ---
//! name: pr-reviewer
//! role: reviewer
//! model: opus
//! tier: 2
//! description: Reviews pull requests for quality and correctness
//! tools: read, bash, github
//! hooks:
//!   pre_tool_use: check-permissions.sh
//! ---
//! # System Prompt
//! You are an expert code reviewer...
//! ```
//!
//! The markdown body becomes the agent's system prompt.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A hook entry in an agent's frontmatter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHookEntry {
    pub event: String,
    pub handler: String,
    pub blocking: bool,
}

/// An agent definition parsed from a markdown file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub name: String,
    pub role: String,
    pub model: String,
    pub tier: u8,
    pub description: String,
    /// Tools this agent is allowed to invoke
    pub tools: Vec<String>,
    /// Hook handlers declared in frontmatter
    pub hooks: Vec<AgentHookEntry>,
    /// The markdown body — used as the system prompt
    pub system_prompt: String,
    /// Raw YAML frontmatter key-value pairs
    pub frontmatter: HashMap<String, String>,
    pub path: PathBuf,
}

impl AgentDefinition {
    /// Parse an agent definition from a markdown string.
    pub fn from_markdown(content: &str, path: &Path) -> Option<Self> {
        let content = content.trim();

        let (fm_str, body) = if content.starts_with("---") {
            let rest = &content[3..];
            let end = rest.find("\n---")?;
            (rest[..end].to_string(), rest[end + 4..].trim().to_string())
        } else {
            return None;
        };

        let mut frontmatter: HashMap<String, String> = HashMap::new();
        for line in fm_str.lines() {
            if let Some((k, v)) = line.split_once(':') {
                frontmatter.insert(k.trim().to_string(), v.trim().to_string());
            }
        }

        let name = frontmatter.get("name").cloned().unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string()
        });

        let tools = frontmatter
            .get("tools")
            .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        Some(Self {
            name,
            role: frontmatter
                .get("role")
                .cloned()
                .unwrap_or_else(|| "assistant".to_string()),
            model: frontmatter
                .get("model")
                .cloned()
                .unwrap_or_else(|| "sonnet".to_string()),
            tier: frontmatter
                .get("tier")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            description: frontmatter.get("description").cloned().unwrap_or_default(),
            tools,
            hooks: Vec::new(), // hook parsing left to registry scan
            system_prompt: body,
            frontmatter,
            path: path.to_path_buf(),
        })
    }

    /// Load from a file on disk.
    pub fn from_file(path: &Path) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        Self::from_markdown(&content, path)
    }
}

/// Registry of agent definitions discovered from markdown files
#[derive(Debug, Default)]
pub struct AgentRegistry {
    agents: HashMap<String, AgentDefinition>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Discover agents from a directory of `*.md` files.
    pub fn discover(dir: &Path) -> Self {
        let mut registry = Self::new();

        let Ok(entries) = std::fs::read_dir(dir) else {
            return registry;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            if let Some(def) = AgentDefinition::from_file(&path) {
                registry.register(def);
            }
        }

        registry
    }

    pub fn register(&mut self, def: AgentDefinition) {
        self.agents.insert(def.name.clone(), def);
    }

    pub fn get(&self, name: &str) -> Option<&AgentDefinition> {
        self.agents.get(name)
    }

    pub fn list(&self) -> Vec<&AgentDefinition> {
        let mut agents: Vec<&AgentDefinition> = self.agents.values().collect();
        agents.sort_by_key(|a| &a.name);
        agents
    }

    pub fn list_by_role(&self, role: &str) -> Vec<&AgentDefinition> {
        let mut agents: Vec<&AgentDefinition> =
            self.agents.values().filter(|a| a.role == role).collect();
        agents.sort_by_key(|a| &a.name);
        agents
    }

    pub fn count(&self) -> usize {
        self.agents.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const SAMPLE: &str = r#"---
name: pr-reviewer
role: reviewer
model: opus
tier: 2
description: Reviews pull requests
tools: read, bash, github
---
# System Prompt

You are an expert code reviewer. Focus on correctness and security.
"#;

    #[test]
    fn test_parse_agent_from_markdown() {
        let path = PathBuf::from("pr-reviewer.md");
        let agent = AgentDefinition::from_markdown(SAMPLE, &path).unwrap();

        assert_eq!(agent.name, "pr-reviewer");
        assert_eq!(agent.role, "reviewer");
        assert_eq!(agent.model, "opus");
        assert_eq!(agent.tier, 2);
        assert_eq!(agent.tools, vec!["read", "bash", "github"]);
        assert!(agent.system_prompt.contains("code reviewer"));
    }

    #[test]
    fn test_missing_frontmatter_returns_none() {
        let content = "No frontmatter here";
        let path = PathBuf::from("no-fm.md");
        assert!(AgentDefinition::from_markdown(content, &path).is_none());
    }

    #[test]
    fn test_agent_registry_register_and_get() {
        let path = PathBuf::from("pr-reviewer.md");
        let agent = AgentDefinition::from_markdown(SAMPLE, &path).unwrap();

        let mut registry = AgentRegistry::new();
        registry.register(agent);

        assert_eq!(registry.count(), 1);
        let got = registry.get("pr-reviewer").unwrap();
        assert_eq!(got.model, "opus");
    }

    #[test]
    fn test_list_by_role() {
        let mut registry = AgentRegistry::new();

        for (name, role) in [
            ("alpha", "reviewer"),
            ("beta", "writer"),
            ("gamma", "reviewer"),
        ] {
            registry.register(AgentDefinition {
                name: name.to_string(),
                role: role.to_string(),
                model: "sonnet".to_string(),
                tier: 0,
                description: String::new(),
                tools: vec![],
                hooks: vec![],
                system_prompt: String::new(),
                frontmatter: HashMap::new(),
                path: PathBuf::from(format!("{}.md", name)),
            });
        }

        let reviewers = registry.list_by_role("reviewer");
        assert_eq!(reviewers.len(), 2);
        assert_eq!(reviewers[0].name, "alpha");
        assert_eq!(reviewers[1].name, "gamma");
    }

    #[test]
    fn test_defaults_when_fields_absent() {
        let minimal = "---\nname: minimal-agent\n---\nDo things.";
        let path = PathBuf::from("minimal-agent.md");
        let agent = AgentDefinition::from_markdown(minimal, &path).unwrap();
        assert_eq!(agent.role, "assistant");
        assert_eq!(agent.model, "sonnet");
        assert_eq!(agent.tier, 0);
        assert!(agent.tools.is_empty());
    }
}
