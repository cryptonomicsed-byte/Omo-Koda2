use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSettings {
    pub plugin_name: String,
    #[serde(flatten)]
    data: HashMap<String, String>,
}

impl PluginSettings {
    pub fn new(plugin_name: impl Into<String>) -> Self {
        Self {
            plugin_name: plugin_name.into(),
            data: HashMap::new(),
        }
    }

    pub fn load_from(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read settings file: {e}"))?;
        parse_settings_from_markdown(&content)
    }

    pub fn save_to(&self, path: &Path) -> Result<(), String> {
        let yaml_data: serde_yaml::Value = {
            let mut map = serde_yaml::Mapping::new();
            map.insert(
                serde_yaml::Value::String("plugin_name".to_string()),
                serde_yaml::Value::String(self.plugin_name.clone()),
            );
            for (k, v) in &self.data {
                map.insert(
                    serde_yaml::Value::String(k.clone()),
                    serde_yaml::Value::String(v.clone()),
                );
            }
            serde_yaml::Value::Mapping(map)
        };

        let yaml_str = serde_yaml::to_string(&yaml_data)
            .map_err(|e| format!("YAML serialize error: {e}"))?;

        let content = format!("---\n{}---\n", yaml_str);

        // Atomic write: temp file then rename to avoid partial reads
        let parent = path
            .parent()
            .ok_or("path has no parent directory")?;
        let tmp_path = parent.join(format!(
            ".{}.tmp",
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("settings")
        ));

        std::fs::write(&tmp_path, &content)
            .map_err(|e| format!("write temp file failed: {e}"))?;
        std::fs::rename(&tmp_path, path)
            .map_err(|e| format!("rename failed: {e}"))?;

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.data.insert(key.into(), value.into());
    }

    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.data.remove(key)
    }
}

fn parse_settings_from_markdown(content: &str) -> Result<PluginSettings, String> {
    let stripped = content.trim_start();
    let frontmatter = if stripped.starts_with("---") {
        let after_open = &stripped[3..];
        let close = after_open
            .find("\n---")
            .ok_or("unclosed frontmatter fence")?;
        after_open[..close].trim().to_string()
    } else {
        return Err("settings file must begin with YAML frontmatter".to_string());
    };

    #[derive(Deserialize)]
    struct RawSettings {
        plugin_name: Option<String>,
        #[serde(flatten)]
        extra: HashMap<String, serde_yaml::Value>,
    }

    let raw: RawSettings = serde_yaml::from_str(&frontmatter)
        .map_err(|e| format!("YAML parse error: {e}"))?;

    let plugin_name = raw.plugin_name.unwrap_or_default();
    let data = raw
        .extra
        .into_iter()
        .filter_map(|(k, v)| {
            let s = match v {
                serde_yaml::Value::String(s) => s,
                serde_yaml::Value::Bool(b) => b.to_string(),
                serde_yaml::Value::Number(n) => n.to_string(),
                _ => return None,
            };
            Some((k, s))
        })
        .collect();

    Ok(PluginSettings { plugin_name, data })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_save_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("my-plugin.local.md");

        let mut settings = PluginSettings::new("my-plugin");
        settings.set("api_endpoint", "https://node.omokoda.ai");
        settings.set("reputation_tier", "3");

        settings.save_to(&path).unwrap();

        let loaded = PluginSettings::load_from(&path).unwrap();
        assert_eq!(loaded.plugin_name, "my-plugin");
        assert_eq!(loaded.get("api_endpoint"), Some("https://node.omokoda.ai"));
        assert_eq!(loaded.get("reputation_tier"), Some("3"));
    }

    #[test]
    fn get_set_works() {
        let mut settings = PluginSettings::new("wallet-plugin");
        assert_eq!(settings.get("address"), None);
        settings.set("address", "0xABC");
        assert_eq!(settings.get("address"), Some("0xABC"));
        let removed = settings.remove("address");
        assert_eq!(removed.as_deref(), Some("0xABC"));
        assert_eq!(settings.get("address"), None);
    }

    #[test]
    fn load_missing_file_errors() {
        let result = PluginSettings::load_from(Path::new("/nonexistent/path.md"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot read settings file"));
    }
}
