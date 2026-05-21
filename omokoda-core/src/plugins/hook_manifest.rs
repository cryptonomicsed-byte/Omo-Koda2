use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum HookEvent {
    PreToolUse,
    PostToolUse,
    Stop,
    UserPromptSubmit,
    SessionStart,
    SessionEnd,
    PreCompact,
    Notification,
}

fn default_timeout() -> u64 {
    30_000
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookEntry {
    pub event: HookEvent,
    pub command: String,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub blocking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HookManifestFile {
    pub hooks: Vec<HookEntry>,
}

impl HookManifestFile {
    pub fn from_json(s: &str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| format!("JSON parse error: {e}"))
    }

    pub fn entries_for(&self, event: HookEvent) -> Vec<&HookEntry> {
        self.hooks.iter().filter(|h| h.event == event).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_from_json() {
        let json = r#"{
            "hooks": [
                {"event": "pre_tool_use", "command": "echo pre", "timeout_ms": 5000, "blocking": true},
                {"event": "session_start", "command": "init.sh"}
            ]
        }"#;
        let manifest = HookManifestFile::from_json(json).unwrap();
        assert_eq!(manifest.hooks.len(), 2);
        assert_eq!(manifest.hooks[0].event, HookEvent::PreToolUse);
        assert!(manifest.hooks[0].blocking);
        assert_eq!(manifest.hooks[1].timeout_ms, 30_000);
    }

    #[test]
    fn entries_for_filters_correctly() {
        let manifest = HookManifestFile {
            hooks: vec![
                HookEntry {
                    event: HookEvent::PreToolUse,
                    command: "a".to_string(),
                    timeout_ms: 1000,
                    blocking: false,
                },
                HookEntry {
                    event: HookEvent::Stop,
                    command: "b".to_string(),
                    timeout_ms: 1000,
                    blocking: false,
                },
                HookEntry {
                    event: HookEvent::PreToolUse,
                    command: "c".to_string(),
                    timeout_ms: 1000,
                    blocking: false,
                },
            ],
        };
        let pre = manifest.entries_for(HookEvent::PreToolUse);
        assert_eq!(pre.len(), 2);
        assert_eq!(pre[0].command, "a");
        assert_eq!(pre[1].command, "c");

        let stop = manifest.entries_for(HookEvent::Stop);
        assert_eq!(stop.len(), 1);
    }

    #[test]
    fn empty_manifest_is_valid() {
        let manifest = HookManifestFile::from_json(r#"{"hooks": []}"#).unwrap();
        assert!(manifest.hooks.is_empty());
        assert!(manifest.entries_for(HookEvent::SessionStart).is_empty());
    }
}
