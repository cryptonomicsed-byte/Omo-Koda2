use crate::plugins::manifest::PluginManifest;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct DiscoveredPlugin {
    pub path: PathBuf,
    pub manifest: PluginManifest,
}

pub struct PluginDiscovery;

impl PluginDiscovery {
    pub fn scan(root: &Path) -> Vec<DiscoveredPlugin> {
        Self::scan_with_depth(root, usize::MAX)
    }

    pub fn scan_with_depth(root: &Path, max_depth: usize) -> Vec<DiscoveredPlugin> {
        WalkDir::new(root)
            .max_depth(max_depth)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|e| {
                e.file_type().is_dir() && e.file_name().to_string_lossy() == ".omokoda-plugin"
            })
            .filter_map(|dir| {
                let manifest_path = dir.path().join("plugin.json");
                let content = std::fs::read_to_string(&manifest_path).ok()?;
                let manifest = PluginManifest::from_json(&content).ok()?;
                Some(DiscoveredPlugin {
                    path: dir.path().to_path_buf(),
                    manifest,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn make_manifest_json(name: &str) -> String {
        format!(
            r#"{{
                "name": "{name}",
                "version": "0.1.0",
                "description": "Test plugin",
                "author": "test",
                "permissions": [],
                "type": "external",
                "tools": [],
                "hooks": [],
                "lifecycle": {{}},
                "min_tier": 0
            }}"#,
        )
    }

    #[test]
    fn scan_finds_plugin_in_subdirectory() {
        let root = tempfile::tempdir().unwrap();
        let plugin_dir = root.path().join("sub").join(".omokoda-plugin");
        std::fs::create_dir_all(&plugin_dir).unwrap();
        std::fs::write(
            plugin_dir.join("plugin.json"),
            make_manifest_json("test-plugin"),
        )
        .unwrap();

        let found = PluginDiscovery::scan(root.path());
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].manifest.name, "test-plugin");
    }

    #[test]
    fn scan_returns_empty_for_dir_without_plugins() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("some").join("nested")).unwrap();

        let found = PluginDiscovery::scan(root.path());
        assert!(found.is_empty());
    }
}
