//! Node-to-node capture delegation via A2A v1.0.
//!
//! Copied from sovereign-stack/sovereign-node/src/delegation.rs (P2 migration).
//! Uses a direct reqwest HTTP call to the peer's /a2a/tasks endpoint rather
//! than the sovereign_a2a crate (not a dependency of omokoda-core).
//!
//! Flow:
//!   1. POST /capture/delegate  { device_id, hint?, peer? }
//!   2. Pick a peer (by name if specified, else first available)
//!   3. Submit A2A task to peer's /a2a/tasks endpoint
//!   4. Return task_id + poll URL so caller can track progress

use serde::{Deserialize, Serialize};

/// Peer node configuration — defined inline (no crate dependency).
pub struct PeerNodeConfig {
    pub name:          String,
    pub a2a_base_url:  String,
}

/// POST /capture/delegate request body.
#[derive(Debug, Deserialize)]
pub struct DelegateRequest {
    pub device_id: String,
    #[serde(default)]
    pub hint: String,
    /// Prefer a specific peer by name; if omitted, first configured peer is used.
    pub peer: Option<String>,
}

/// Result of a successful delegation.
#[derive(Debug, Serialize)]
pub struct DelegateResult {
    pub peer_name: String,
    pub peer_url:  String,
    pub task_id:   String,
    pub poll_url:  String,
}

/// A minimal A2A task submission body.
#[derive(Debug, Serialize)]
struct A2aTaskRequest {
    message: A2aMessage,
}

#[derive(Debug, Serialize)]
struct A2aMessage {
    role:  String,
    parts: Vec<A2aPart>,
}

#[derive(Debug, Serialize)]
struct A2aPart {
    r#type: String,
    text:   String,
}

/// Minimal task response — only the id field is required.
#[derive(Debug, Deserialize)]
struct A2aTaskResponse {
    id: String,
}

/// Delegate a capture task to the best available peer node.
pub async fn delegate_capture(
    peers:     &[PeerNodeConfig],
    device_id: &str,
    hint:      &str,
    prefer:    Option<&str>,
) -> Result<DelegateResult, String> {
    let peer = match prefer {
        Some(name) => peers.iter()
            .find(|p| p.name == name)
            .ok_or_else(|| format!("peer '{name}' not found in config"))?,
        None => peers.first()
            .ok_or_else(|| "no peers configured — add [[peers.nodes]] to config.toml".to_string())?,
    };

    let text = if hint.is_empty() {
        format!("capture {device_id}")
    } else {
        format!("capture {device_id} {hint}")
    };

    let body = A2aTaskRequest {
        message: A2aMessage {
            role:  "user".into(),
            parts: vec![A2aPart { r#type: "text".into(), text }],
        },
    };

    let url = format!("{}/a2a/tasks", peer.a2a_base_url);
    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("A2A submit failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("A2A submit HTTP {}", resp.status()));
    }

    let task: A2aTaskResponse = resp
        .json()
        .await
        .map_err(|e| format!("A2A response parse failed: {e}"))?;

    Ok(DelegateResult {
        poll_url:  format!("{}/a2a/tasks/{}", peer.a2a_base_url, task.id),
        peer_name: peer.name.clone(),
        peer_url:  peer.a2a_base_url.clone(),
        task_id:   task.id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delegate_request_deserialises() {
        let json = r#"{"device_id":"unitree:go2:1","hint":"","peer":null}"#;
        let req: DelegateRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.device_id, "unitree:go2:1");
        assert!(req.peer.is_none());
    }

    #[test]
    fn pick_peer_by_name() {
        let peers = vec![
            PeerNodeConfig { name: "alpha".into(), a2a_base_url: "http://alpha:7779".into() },
            PeerNodeConfig { name: "beta".into(),  a2a_base_url: "http://beta:7779".into() },
        ];
        let found = peers.iter().find(|p| p.name == "beta").unwrap();
        assert_eq!(found.a2a_base_url, "http://beta:7779");
    }

    #[test]
    fn delegate_text_combines_device_hint() {
        let hint = "panoramic";
        let device_id = "unitree:go2:192.168.1.10";
        let text = format!("capture {device_id} {hint}");
        assert!(text.starts_with("capture unitree:go2:"));
        assert!(text.contains("panoramic"));
    }

    #[tokio::test]
    async fn delegate_fails_gracefully_with_no_peers() {
        let result = delegate_capture(&[], "go2:1", "", None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no peers configured"));
    }

    #[tokio::test]
    async fn delegate_fails_gracefully_with_unknown_peer() {
        let peers = vec![
            PeerNodeConfig { name: "alpha".into(), a2a_base_url: "http://alpha:7779".into() },
        ];
        let result = delegate_capture(&peers, "go2:1", "", Some("gamma")).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }
}
