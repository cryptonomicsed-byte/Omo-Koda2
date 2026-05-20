//! Plugin manifest definition — describes a plugin's capabilities and requirements

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    /// Required permissions (e.g. "read:workspace/*", "exec:*")
    pub permissions: Vec<String>,
    /// Hook configurations
    pub hooks: Vec<HookConfig>,
    /// Tool definitions provided by this plugin
    pub tools: Vec<PluginToolConfig>,
    /// Lifecycle commands
    pub lifecycle: LifecycleConfig,
    /// Minimum agent tier required
    pub min_tier: u8,
    /// Plugin type
    #[serde(rename = "type")]
    pub plugin_type: PluginType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginType {
    Bundled,     // Shipped with Omo-Koda
    External,    // Installed from git/local path
    Marketplace, // From Garden Marketplace
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    /// When this hook fires: "pre_tool_use", "post_tool_use", "on_think", etc.
    pub event: String,
    /// Command to execute (receives JSON context on stdin)
    pub command: String,
    /// Whether hook failure blocks execution
    pub blocking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginToolConfig {
    pub name: String,
    pub description: String,
    /// Path to executable
    pub command: String,
    /// Required tier
    pub required_tier: u8,
    /// Whether this is a write operation
    pub is_write: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LifecycleConfig {
    /// Command to run on plugin init
    pub init: Option<String>,
    /// Command to run on plugin shutdown
    pub shutdown: Option<String>,
}

impl PluginManifest {
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("Invalid plugin manifest: {}", e))
    }

    pub fn from_file(path: &std::path::Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;
        Self::from_json(&content)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Plugin name cannot be empty".to_string());
        }
        if self.version.is_empty() {
            return Err("Plugin version cannot be empty".to_string());
        }
        // Validate permission patterns
        for perm in &self.permissions {
            if !perm.contains(':') {
                return Err(format!(
                    "Invalid permission format: '{}' (expected 'action:resource')",
                    perm
                ));
            }
        }
        Ok(())
    }
}
