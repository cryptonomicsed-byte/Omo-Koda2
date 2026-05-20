//! Skill/Odu module discovery — hierarchical knowledge loading with shadowing.
//! Ports Claw-code's skill loading pattern.
//!
//! Search order (highest priority first):
//!   1. Agent-local: .omokoda/agents/<id>/skills/
//!   2. Project: .omokoda/skills/ (cwd)
//!   3. Global: ~/.omokoda/skills/
//!   4. Marketplace: ~/.omokoda/marketplace/skills/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OduSource {
    AgentLocal,
    Project,
    Global,
    Marketplace,
}

impl std::fmt::Display for OduSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AgentLocal => write!(f, "agent-local"),
            Self::Project => write!(f, "project"),
            Self::Global => write!(f, "global"),
            Self::Marketplace => write!(f, "marketplace"),
        }
    }
}

/// An Odu module — a knowledge/skill bundle loaded from markdown with frontmatter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OduModule {
    pub name: String,
    pub version: String,
    pub description: String,
    pub source: OduSource,
    pub path: PathBuf,
    /// YAML frontmatter key-value pairs
    pub frontmatter: HashMap<String, String>,
    /// The markdown body content
    pub body: String,
    /// Required tier to use this module
    pub required_tier: u8,
    /// Invocation description (how to trigger this skill)
    pub invocation: String,
}

/// Registry of discovered Odu modules with shadowing
pub struct OduRegistry {
    pub(crate) modules: HashMap<String, OduModule>,
}

impl OduRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    /// Discover modules from all search roots, with priority-based shadowing
    pub fn discover(agent_id: &str, cwd: &Path) -> Self {
        let mut registry = Self::new();

        // Search roots in REVERSE priority order (lower priority loaded first, higher overrides)
        // So: Marketplace → Global → Project → AgentLocal
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_default();

        let search_roots = vec![
            (
                home.join(".omokoda").join("marketplace").join("skills"),
                OduSource::Marketplace,
            ),
            (home.join(".omokoda").join("skills"), OduSource::Global),
            (cwd.join(".omokoda").join("skills"), OduSource::Project),
            (
                cwd.join(".omokoda")
                    .join("agents")
                    .join(agent_id)
                    .join("skills"),
                OduSource::AgentLocal,
            ),
        ];

        for (root, source) in search_roots {
            if root.exists() {
                for module in scan_modules(&root, source) {
                    // Higher-priority sources override lower-priority (HashMap::insert)
                    registry.modules.insert(module.name.clone(), module);
                }
            }
        }

        registry
    }

    pub fn get(&self, name: &str) -> Option<&OduModule> {
        self.modules.get(name)
    }

    pub fn list(&self) -> Vec<&OduModule> {
        let mut modules: Vec<&OduModule> = self.modules.values().collect();
        modules.sort_by_key(|m| &m.name);
        modules
    }

    pub fn search(&self, query: &str) -> Vec<&OduModule> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<&OduModule> = self
            .modules
            .values()
            .filter(|m| {
                m.name.to_lowercase().contains(&query_lower)
                    || m.description.to_lowercase().contains(&query_lower)
                    || m.invocation.to_lowercase().contains(&query_lower)
            })
            .collect();
        results.sort_by_key(|m| &m.name);
        results
    }

    pub fn count(&self) -> usize {
        self.modules.len()
    }
}

impl Default for OduRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Scan a directory for .md skill files
fn scan_modules(root: &Path, source: OduSource) -> Vec<OduModule> {
    let mut modules = Vec::new();

    let entries = match std::fs::read_dir(root) {
        Ok(e) => e,
        Err(_) => return modules,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Some(module) = parse_odu_module(&path, &content, source.clone()) {
                modules.push(module);
            }
        }
    }

    modules
}

/// Parse an Odu module from markdown with YAML frontmatter
/// Format:
/// ```text
/// ---
/// name: my-skill
/// version: 1.0.0
/// description: Does something useful
/// tier: 1
/// invocation: /my-skill <arg>
/// ---
/// # Skill Body
/// ...
/// ```
fn parse_odu_module(path: &Path, content: &str, source: OduSource) -> Option<OduModule> {
    let content = content.trim();

    // Extract frontmatter between --- delimiters
    let (frontmatter_str, body) = if content.starts_with("---") {
        let rest = &content[3..];
        if let Some(end) = rest.find("\n---") {
            let fm = &rest[..end];
            let body = rest[end + 4..].trim().to_string();
            (fm.to_string(), body)
        } else {
            return None;
        }
    } else {
        // No frontmatter — use filename as name
        ("".to_string(), content.to_string())
    };

    let mut frontmatter: HashMap<String, String> = HashMap::new();
    for line in frontmatter_str.lines() {
        if let Some((key, value)) = line.split_once(':') {
            frontmatter.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    // Derive name: frontmatter > filename stem
    let name = frontmatter.get("name").cloned().unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    });

    let version = frontmatter
        .get("version")
        .cloned()
        .unwrap_or_else(|| "0.1.0".to_string());
    let description = frontmatter
        .get("description")
        .cloned()
        .unwrap_or_else(|| name.clone());
    let invocation = frontmatter
        .get("invocation")
        .cloned()
        .unwrap_or_else(|| format!("/{}", name));
    let required_tier: u8 = frontmatter
        .get("tier")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    Some(OduModule {
        name,
        version,
        description,
        source,
        path: path.to_path_buf(),
        frontmatter,
        body,
        required_tier,
        invocation,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_odu_module() {
        let content = r#"---
name: test-skill
version: 1.0.0
description: A test skill
tier: 1
invocation: /test-skill <arg>
---
# Test Skill

This skill does something useful.
"#;
        let path = PathBuf::from("/tmp/test-skill.md");
        let module = parse_odu_module(&path, content, OduSource::Project).unwrap();
        assert_eq!(module.name, "test-skill");
        assert_eq!(module.version, "1.0.0");
        assert_eq!(module.required_tier, 1);
        assert!(module.body.contains("This skill does something useful"));
    }

    #[test]
    fn test_odu_registry_search() {
        let mut registry = OduRegistry::new();
        registry.modules.insert(
            "read-files".to_string(),
            OduModule {
                name: "read-files".to_string(),
                version: "1.0.0".to_string(),
                description: "Read files from workspace".to_string(),
                source: OduSource::Project,
                path: PathBuf::from("/tmp/read-files.md"),
                frontmatter: HashMap::new(),
                body: "".to_string(),
                required_tier: 0,
                invocation: "/read".to_string(),
            },
        );

        let results = registry.search("read");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "read-files");

        let empty = registry.search("xyz-nonexistent");
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_odu_source_display() {
        assert_eq!(OduSource::AgentLocal.to_string(), "agent-local");
        assert_eq!(OduSource::Project.to_string(), "project");
        assert_eq!(OduSource::Global.to_string(), "global");
        assert_eq!(OduSource::Marketplace.to_string(), "marketplace");
    }
}
