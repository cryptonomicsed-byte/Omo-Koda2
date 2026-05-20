//! System prompt construction — assembles the agent's context into a structured system prompt.
//!
//! Ports Claw-code's prompt.rs pattern: inject cwd, git status, date, OS, available tools,
//! agent identity, and instructions into a coherent system prompt.

use crate::config::FeatureFlags;
use crate::identity::odu::OduIdentity;
use crate::identity::AgentId;
use std::path::PathBuf;

/// Builds the system prompt for an agent's think cycle
pub struct SystemPromptBuilder {
    pub agent_name: String,
    pub agent_id: AgentId,
    pub tier: u8,
    pub reputation: f64,
    pub odu_identity: OduIdentity,
    pub workspace_root: PathBuf,
    pub feature_flags: FeatureFlags,
    pub available_tools: Vec<String>,
    pub custom_instructions: Vec<String>,
}

impl SystemPromptBuilder {
    pub fn new(
        agent_name: &str,
        agent_id: AgentId,
        tier: u8,
        reputation: f64,
        odu_identity: OduIdentity,
        workspace_root: PathBuf,
    ) -> Self {
        Self {
            agent_name: agent_name.to_string(),
            agent_id,
            tier,
            reputation,
            odu_identity,
            workspace_root,
            feature_flags: FeatureFlags::default(),
            available_tools: Vec::new(),
            custom_instructions: Vec::new(),
        }
    }

    pub fn with_feature_flags(mut self, flags: FeatureFlags) -> Self {
        self.feature_flags = flags;
        self
    }

    pub fn with_tools(mut self, tools: Vec<String>) -> Self {
        self.available_tools = tools;
        self
    }

    pub fn with_instructions(mut self, instructions: Vec<String>) -> Self {
        self.custom_instructions = instructions;
        self
    }

    /// Build the complete system prompt
    pub fn build(&self) -> String {
        let mut sections = Vec::new();

        // Identity section
        sections.push(self.identity_section());

        // Environment section
        sections.push(self.environment_section());

        // Git status (if available)
        if let Some(git) = self.git_status_section() {
            sections.push(git);
        }

        // Available tools
        if !self.available_tools.is_empty() {
            sections.push(self.tools_section());
        }

        // Hermetic principles reminder
        sections.push(self.hermetic_principles_section());

        // Custom instructions
        for instruction in &self.custom_instructions {
            sections.push(instruction.clone());
        }

        sections.join("\n\n")
    }

    fn identity_section(&self) -> String {
        format!(
            "You are {}, a sovereign agent in the Omo-Koda network.\n\
             Agent ID: {}\n\
             Tier: {} | Reputation: {:.1}\n\
             Odu: {} ({})",
            self.agent_name,
            self.agent_id,
            self.tier,
            self.reputation,
            self.odu_identity.primary_index,
            self.odu_identity
                .mnemonic
                .split_whitespace()
                .next()
                .unwrap_or("unknown"),
        )
    }

    fn environment_section(&self) -> String {
        let date = current_date_str();
        let os = std::env::consts::OS;
        let cwd = self.workspace_root.display();

        format!(
            "Environment:\n\
             Date: {}\n\
             OS: {}\n\
             Working directory: {}",
            date, os, cwd
        )
    }

    fn git_status_section(&self) -> Option<String> {
        let output = std::process::Command::new("git")
            .args([
                "-C",
                self.workspace_root.to_str()?,
                "status",
                "--short",
                "--branch",
            ])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let status = String::from_utf8_lossy(&output.stdout).to_string();
        if status.trim().is_empty() {
            return None;
        }

        // Truncate to first 10 lines
        let truncated: String = status.lines().take(10).collect::<Vec<_>>().join("\n");
        Some(format!("Git status:\n```\n{}\n```", truncated))
    }

    fn tools_section(&self) -> String {
        let tool_list = self
            .available_tools
            .iter()
            .map(|t| format!("  - {}", t))
            .collect::<Vec<_>>()
            .join("\n");
        format!("Available tools (tier {} access):\n{}", self.tier, tool_list)
    }

    fn hermetic_principles_section(&self) -> String {
        "Core principles: Correspondence (thought \u{2194} action alignment), \
         Cause & Effect (all acts generate receipts), \
         Rhythm (respect cooldowns and sabbath cycles). \
         Never violate workspace boundaries. \
         Private thoughts stay private."
            .to_string()
    }
}

fn current_date_str() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Simple date formatting without chrono dependency
    let days = secs / 86400;
    let epoch_year = 1970u64;
    // Approximate: days since epoch → date
    let year = epoch_year + days / 365;
    format!("~{} CE", year) // Approximate; precise formatting needs chrono
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::odu::OduIdentity;
    use crate::identity::AgentId;
    use std::path::PathBuf;

    fn make_builder() -> SystemPromptBuilder {
        SystemPromptBuilder::new(
            "Omo",
            AgentId::new("test-fingerprint-1234567890abcdef"),
            1,
            42.0,
            OduIdentity {
                primary_index: 3,
                mnemonic: "Ogunda speaks truth".to_string(),
            },
            PathBuf::from("/tmp"),
        )
    }

    #[test]
    fn test_build_contains_identity() {
        let prompt = make_builder().build();
        assert!(prompt.contains("Omo"));
        assert!(prompt.contains("agent-test-fingerp"));
        assert!(prompt.contains("Tier: 1"));
    }

    #[test]
    fn test_build_contains_environment() {
        let prompt = make_builder().build();
        assert!(prompt.contains("Environment:"));
        assert!(prompt.contains("Working directory:"));
    }

    #[test]
    fn test_build_with_tools() {
        let builder = make_builder().with_tools(vec!["bash".to_string(), "read".to_string()]);
        let prompt = builder.build();
        assert!(prompt.contains("bash"));
        assert!(prompt.contains("read"));
    }

    #[test]
    fn test_build_with_custom_instructions() {
        let builder =
            make_builder().with_instructions(vec!["Always be helpful.".to_string()]);
        let prompt = builder.build();
        assert!(prompt.contains("Always be helpful."));
    }

    #[test]
    fn test_hermetic_principles_present() {
        let prompt = make_builder().build();
        assert!(prompt.contains("Correspondence"));
        assert!(prompt.contains("Cause & Effect"));
    }
}
