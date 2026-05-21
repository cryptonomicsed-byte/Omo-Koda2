//! Plugin system — manage, load, and execute plugins.

pub mod agent;
pub mod command;
pub mod config_loader;
pub mod discovery;
pub mod hook_manifest;
pub mod manifest;
pub mod mcp;
pub mod output_style;
pub mod registry;
pub mod rule_engine;
pub mod settings;
pub mod skill;

pub use agent::AgentDef;
pub use command::CommandDef;
pub use config_loader::ConfigLoader;
pub use discovery::{DiscoveredPlugin, PluginDiscovery};
pub use hook_manifest::{HookEntry, HookEvent, HookManifestFile};
pub use manifest::{HookConfig, LifecycleConfig, PluginManifest, PluginToolConfig, PluginType};
pub use mcp::{McpManifest, McpServerConfig, McpTransport};
pub use registry::{InstalledPlugin, PluginRegistry, PluginState};
pub use rule_engine::{Condition, Rule, RuleAction, RuleContext, RuleEngine, RuleOperator, RuleResult};
pub use output_style::{OutputStyle, StyleDirective};
pub use settings::PluginSettings;
pub use skill::{SkillDef, SkillTier};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manifest_parse() {
        let json = r#"{
            "name": "my-plugin",
            "version": "1.0.0",
            "description": "A test plugin",
            "author": "test",
            "permissions": ["read:workspace/*"],
            "hooks": [],
            "tools": [],
            "lifecycle": {},
            "min_tier": 0,
            "type": "external"
        }"#;
        let manifest = PluginManifest::from_json(json).unwrap();
        assert_eq!(manifest.name, "my-plugin");
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_plugin_manifest_invalid_permission() {
        let json = r#"{
            "name": "bad-plugin",
            "version": "1.0.0",
            "description": "",
            "author": "",
            "permissions": ["invalid-no-colon"],
            "hooks": [],
            "tools": [],
            "lifecycle": {},
            "min_tier": 0,
            "type": "bundled"
        }"#;
        let manifest = PluginManifest::from_json(json).unwrap();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_plugin_registry_empty() {
        let tmp = std::env::temp_dir().join("omokoda-test-registry");
        let registry = PluginRegistry::new(tmp);
        assert_eq!(registry.list().len(), 0);
        assert_eq!(registry.list_enabled().len(), 0);
    }
}

