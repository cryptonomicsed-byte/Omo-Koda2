use async_trait::async_trait;

use crate::tools::{ExecutionContext, Tool};
use crate::usage::TokenUsage;

pub struct ConfigReadTool;

#[async_trait]
impl Tool for ConfigReadTool {
    fn name(&self) -> &str {
        "config_read"
    }
    fn description(&self) -> &str {
        "Read a configuration value. Params: JSON {key} where key uses dot-notation \
         (e.g. 'feature_flags.auto_compact')"
    }
    fn required_tier(&self) -> u8 {
        0
    }
    fn is_write_operation(&self) -> bool {
        false
    }

    async fn execute(
        &self,
        params: &str,
        context: &ExecutionContext,
    ) -> Result<(String, TokenUsage), String> {
        let key = if params.starts_with('{') {
            let v: serde_json::Value = serde_json::from_str(params).map_err(|e| e.to_string())?;
            v["key"].as_str().ok_or("missing key")?.to_string()
        } else {
            params.trim().to_string()
        };

        let config = load_config(&context.workspace_root);
        let value =
            get_nested(&config, &key).ok_or_else(|| format!("Config key not found: {}", key))?;

        Ok((value.to_string(), TokenUsage::default()))
    }
}

pub struct ConfigWriteTool;

#[async_trait]
impl Tool for ConfigWriteTool {
    fn name(&self) -> &str {
        "config_write"
    }
    fn description(&self) -> &str {
        "Write a configuration value. Params: JSON {key, value, scope?} where scope is \
         'project' or 'local' (default: 'local')"
    }
    fn required_tier(&self) -> u8 {
        1
    }
    fn is_write_operation(&self) -> bool {
        true
    }

    async fn execute(
        &self,
        params: &str,
        context: &ExecutionContext,
    ) -> Result<(String, TokenUsage), String> {
        let v: serde_json::Value = serde_json::from_str(params)
            .map_err(|e| format!("config_write requires JSON: {}", e))?;

        let key = v["key"].as_str().ok_or("missing key")?;
        let value = v.get("value").ok_or("missing value")?.clone();
        let scope = v["scope"].as_str().unwrap_or("local");

        let config_path = match scope {
            "project" => context
                .workspace_root
                .join(".omokoda")
                .join("settings.json"),
            _ => context
                .workspace_root
                .join(".omokoda")
                .join("settings.local.json"),
        };

        let mut config: serde_json::Value = std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(serde_json::json!({}));

        set_nested(&mut config, key, value.clone());

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config dir: {}", e))?;
        }
        let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
        std::fs::write(&config_path, json).map_err(|e| format!("Failed to write config: {}", e))?;

        Ok((
            format!("Set {}.{} = {}", scope, key, value),
            TokenUsage::default(),
        ))
    }
}

fn load_config(workspace_root: &std::path::Path) -> serde_json::Value {
    let paths = [
        workspace_root.join(".omokoda").join("settings.local.json"),
        workspace_root.join(".omokoda").join("settings.json"),
        workspace_root.join(".omokoda.json"),
    ];
    for path in &paths {
        if let Ok(s) = std::fs::read_to_string(path) {
            if let Ok(v) = serde_json::from_str(&s) {
                return v;
            }
        }
    }
    serde_json::json!({})
}

fn get_nested<'a>(obj: &'a serde_json::Value, key: &str) -> Option<&'a serde_json::Value> {
    let parts: Vec<&str> = key.splitn(2, '.').collect();
    match parts.as_slice() {
        [single] => obj.get(*single),
        [head, tail] => obj.get(*head).and_then(|v| get_nested(v, tail)),
        _ => None,
    }
}

fn set_nested(obj: &mut serde_json::Value, key: &str, value: serde_json::Value) {
    let parts: Vec<&str> = key.splitn(2, '.').collect();
    match parts.as_slice() {
        [single] => {
            if let Some(map) = obj.as_object_mut() {
                map.insert((*single).to_string(), value);
            }
        }
        [head, tail] => {
            if let Some(map) = obj.as_object_mut() {
                let entry = map
                    .entry((*head).to_string())
                    .or_insert(serde_json::json!({}));
                set_nested(entry, tail, value);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_nested() {
        let obj = serde_json::json!({
            "feature_flags": { "auto_compact": true },
            "provider": "ollama"
        });
        assert_eq!(
            get_nested(&obj, "provider"),
            Some(&serde_json::json!("ollama"))
        );
        assert_eq!(
            get_nested(&obj, "feature_flags.auto_compact"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(get_nested(&obj, "nonexistent"), None);
    }

    #[test]
    fn test_set_nested() {
        let mut obj = serde_json::json!({});
        set_nested(
            &mut obj,
            "feature_flags.auto_compact",
            serde_json::json!(false),
        );
        assert_eq!(
            obj["feature_flags"]["auto_compact"],
            serde_json::json!(false)
        );
    }
}
