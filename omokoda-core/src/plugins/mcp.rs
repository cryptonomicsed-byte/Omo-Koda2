use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum McpTransport {
    Stdio,
    Sse { url: String },
    Http { url: String },
    WebSocket { url: String },
}

fn default_timeout() -> u64 {
    30_000
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub transport: McpTransport,
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpManifest {
    pub servers: Vec<McpServerConfig>,
}

impl McpManifest {
    pub fn from_json(s: &str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| format!("JSON parse error: {e}"))
    }

    pub fn find_server(&self, name: &str) -> Option<&McpServerConfig> {
        self.servers.iter().find(|s| s.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_manifest_from_json() {
        let json = r#"{
            "servers": [
                {
                    "name": "filesystem",
                    "transport": {"type": "stdio"},
                    "command": "mcp-server-filesystem",
                    "args": ["/workspace"],
                    "env": {"LOG_LEVEL": "info"},
                    "timeout_ms": 10000
                },
                {
                    "name": "omo-oracle",
                    "transport": {"type": "sse", "url": "https://oracle.omokoda.ai/mcp"},
                    "args": []
                }
            ]
        }"#;

        let manifest = McpManifest::from_json(json).unwrap();
        assert_eq!(manifest.servers.len(), 2);

        let fs = &manifest.servers[0];
        assert_eq!(fs.name, "filesystem");
        assert!(matches!(fs.transport, McpTransport::Stdio));
        assert_eq!(fs.command.as_deref(), Some("mcp-server-filesystem"));
        assert_eq!(fs.env.get("LOG_LEVEL").map(|s| s.as_str()), Some("info"));

        let oracle = &manifest.servers[1];
        assert_eq!(oracle.timeout_ms, 30_000);
        assert!(matches!(&oracle.transport, McpTransport::Sse { url } if url.contains("omokoda")));
    }

    #[test]
    fn find_server_returns_correct_entry() {
        let manifest = McpManifest::from_json(
            r#"{
            "servers": [
                {"name": "alpha", "transport": {"type": "stdio"}},
                {"name": "beta", "transport": {"type": "http", "url": "http://localhost:8080"}}
            ]
        }"#,
        )
        .unwrap();

        assert!(manifest.find_server("alpha").is_some());
        assert!(matches!(
            manifest.find_server("beta").map(|s| &s.transport),
            Some(McpTransport::Http { url }) if url.contains("8080")
        ));
        assert!(manifest.find_server("gamma").is_none());
    }
}
