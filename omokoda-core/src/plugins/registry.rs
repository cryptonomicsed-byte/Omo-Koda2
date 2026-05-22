//! Plugin registry — manages installed plugins lifecycle and state

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::plugins::manifest::PluginManifest;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PluginState {
    Enabled,
    Disabled,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPlugin {
    pub manifest: PluginManifest,
    pub state: PluginState,
    pub install_path: PathBuf,
    pub installed_at: u64,
    pub source_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PluginIndex {
    plugins: HashMap<String, InstalledPlugin>,
}

/// Manages the lifecycle of installed plugins
pub struct PluginRegistry {
    registry_path: PathBuf,
    index: PluginIndex,
}

impl PluginRegistry {
    pub fn new(registry_path: PathBuf) -> Self {
        let index = Self::load_index(&registry_path);
        Self {
            registry_path,
            index,
        }
    }

    fn load_index(path: &Path) -> PluginIndex {
        let index_path = path.join("plugins.json");
        std::fs::read_to_string(index_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save_index(&self) -> Result<(), String> {
        std::fs::create_dir_all(&self.registry_path)
            .map_err(|e| format!("Failed to create registry dir: {}", e))?;
        let index_path = self.registry_path.join("plugins.json");
        let json = serde_json::to_string_pretty(&self.index)
            .map_err(|e| format!("Failed to serialize index: {}", e))?;
        std::fs::write(index_path, json).map_err(|e| format!("Failed to write index: {}", e))
    }

    /// Install a plugin from a local path
    pub fn install_local(&mut self, source_path: &Path) -> Result<String, String> {
        let manifest_path = source_path.join("plugin.json");
        let manifest = PluginManifest::from_file(&manifest_path)?;
        manifest.validate()?;

        let name = manifest.name.clone();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let plugin = InstalledPlugin {
            manifest,
            state: PluginState::Enabled,
            install_path: source_path.to_path_buf(),
            installed_at: now,
            source_url: source_path.display().to_string(),
        };

        self.index.plugins.insert(name.clone(), plugin);
        self.save_index()?;

        Ok(name)
    }

    /// Enable a plugin
    pub fn enable(&mut self, name: &str) -> Result<(), String> {
        let plugin = self
            .index
            .plugins
            .get_mut(name)
            .ok_or_else(|| format!("Plugin '{}' not found", name))?;
        plugin.state = PluginState::Enabled;
        self.save_index()
    }

    /// Disable a plugin (doesn't uninstall)
    pub fn disable(&mut self, name: &str) -> Result<(), String> {
        let plugin = self
            .index
            .plugins
            .get_mut(name)
            .ok_or_else(|| format!("Plugin '{}' not found", name))?;
        plugin.state = PluginState::Disabled;
        self.save_index()
    }

    /// Uninstall a plugin
    pub fn uninstall(&mut self, name: &str) -> Result<(), String> {
        self.index
            .plugins
            .remove(name)
            .ok_or_else(|| format!("Plugin '{}' not found", name))?;
        self.save_index()
    }

    /// List all installed plugins
    pub fn list(&self) -> Vec<&InstalledPlugin> {
        let mut plugins: Vec<&InstalledPlugin> = self.index.plugins.values().collect();
        plugins.sort_by_key(|p| &p.manifest.name);
        plugins
    }

    /// List enabled plugins
    pub fn list_enabled(&self) -> Vec<&InstalledPlugin> {
        self.list()
            .into_iter()
            .filter(|p| p.state == PluginState::Enabled)
            .collect()
    }

    /// Get a specific plugin
    pub fn get(&self, name: &str) -> Option<&InstalledPlugin> {
        self.index.plugins.get(name)
    }

    /// Get all tool configs from enabled plugins
    pub fn active_tools(&self) -> Vec<(&str, &crate::plugins::manifest::PluginToolConfig)> {
        self.list_enabled()
            .iter()
            .flat_map(|p| {
                p.manifest
                    .tools
                    .iter()
                    .map(move |t| (p.manifest.name.as_str(), t))
            })
            .collect()
    }

    /// Get all hook configs from enabled plugins
    pub fn active_hooks(&self) -> Vec<(&str, &crate::plugins::manifest::HookConfig)> {
        self.list_enabled()
            .iter()
            .flat_map(|p| {
                p.manifest
                    .hooks
                    .iter()
                    .map(move |h| (p.manifest.name.as_str(), h))
            })
            .collect()
    }
}
