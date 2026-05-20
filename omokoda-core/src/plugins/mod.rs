//! Plugin system — manage, load, and execute plugins.
//! Ports Claw-code's plugin pattern (#15-20).

pub mod manifest;
pub mod registry;

pub use manifest::{HookConfig, LifecycleConfig, PluginManifest, PluginToolConfig, PluginType};
pub use registry::{InstalledPlugin, PluginRegistry, PluginState};

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
