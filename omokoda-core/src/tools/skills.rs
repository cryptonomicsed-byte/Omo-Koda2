//! Config-driven external service skills.
//!
//! Rather than hand-writing a Rust [`Tool`] per external service, a service is
//! described by a **manifest entry** — a base URL, auth, and a map of named
//! routes — and exposed as a single generic [`ExternalServiceTool`]. Adding a
//! new project becomes a manifest entry, not a recompile.
//!
//! A skill is invoked through the normal `act` primitive:
//!
//! ```text
//! act vantage {"route":"block_agents","path":{"block_id":"default"}}
//! ```
//!
//! The [`SkillsListTool`] (`skills`) lists every registered service skill and
//! its routes. Everything inherits the registry's tier-gating, permissions, and
//! receipts for free — there is no parallel registry.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::tools::{ExecutionContext, Tool};
use crate::usage::TokenUsage;

static HTTP: OnceLock<Client> = OnceLock::new();
fn http() -> &'static Client {
    HTTP.get_or_init(Client::new)
}

fn default_tier() -> u8 {
    1
}

/// One external service, declared in a manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillManifestEntry {
    /// Skill name — the `act <name>` handle.
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// Base URL, may contain `${ENV}` references resolved at call time.
    pub base_url: String,
    /// Header to carry the auth token (e.g. `X-Agent-Key`).
    #[serde(default)]
    pub auth_header: Option<String>,
    /// Env var holding the auth token (e.g. `VANTAGE_KEY`).
    #[serde(default)]
    pub auth_env: Option<String>,
    #[serde(default = "default_tier")]
    pub required_tier: u8,
    #[serde(default)]
    pub write: bool,
    /// Route name → `"METHOD /path/{param}"`.
    pub routes: HashMap<String, String>,
}

/// A set of service skills, loadable from a JSON manifest file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillManifest {
    #[serde(default)]
    pub skills: Vec<SkillManifestEntry>,
}

/// The built-in manifest. Ships with **Vantage** as a worked example so the
/// adapter has a live API to exercise out of the box.
pub fn default_manifest() -> SkillManifest {
    let routes = [
        ("block_snapshot", "GET /api/mesh/blocks/{block_id}"),
        ("block_agents", "GET /api/mesh/blocks/{block_id}/agents"),
        ("resources", "GET /api/mesh/resources/{block_id}"),
        ("trust", "GET /api/mesh/trust/{agent_id}"),
        ("signal", "POST /api/mesh/signal"),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();

    SkillManifest {
        skills: vec![SkillManifestEntry {
            name: "vantage".to_string(),
            description: "Vantage mesh API — block snapshots, agents, resources, trust, signals"
                .to_string(),
            base_url: "${VANTAGE_URL}".to_string(),
            auth_header: Some("X-Agent-Key".to_string()),
            auth_env: Some("VANTAGE_KEY".to_string()),
            required_tier: 1,
            write: false,
            routes,
        }],
    }
}

fn val_to_str(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// Resolve `${VAR}` references in `s` from the environment. Returns `None` if any
/// referenced variable is missing or empty — the signal that a skill is not
/// configured on this runtime.
pub fn resolve_env(s: &str) -> Option<String> {
    let mut out = String::new();
    let mut rest = s;
    while let Some(start) = rest.find("${") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let end = after.find('}')?;
        let val = std::env::var(&after[..end]).ok()?;
        if val.is_empty() {
            return None;
        }
        out.push_str(&val);
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Some(out)
}

/// Build the `(method, url)` for a route invocation. Pure — no I/O — so the
/// routing logic is testable without a server.
pub fn build_invocation(
    route_str: &str,
    base: &str,
    params: &serde_json::Value,
) -> Result<(String, String), String> {
    let (method, path_tmpl) = route_str
        .split_once(' ')
        .ok_or_else(|| format!("malformed route '{route_str}'"))?;
    let method = method.to_ascii_uppercase();

    let mut path = path_tmpl.to_string();
    if let Some(obj) = params.get("path").and_then(|v| v.as_object()) {
        for (k, v) in obj {
            let encoded = urlencoding::encode(&val_to_str(v)).into_owned();
            path = path.replace(&format!("{{{k}}}"), &encoded);
        }
    }
    if path.contains('{') {
        return Err(format!("unfilled path parameter in '{path}'"));
    }

    let mut url = format!("{}{}", base.trim_end_matches('/'), path);
    if let Some(obj) = params.get("query").and_then(|v| v.as_object()) {
        let qs: Vec<String> = obj
            .iter()
            .map(|(k, v)| {
                format!(
                    "{}={}",
                    urlencoding::encode(k),
                    urlencoding::encode(&val_to_str(v))
                )
            })
            .collect();
        if !qs.is_empty() {
            url.push('?');
            url.push_str(&qs.join("&"));
        }
    }
    Ok((method, url))
}

/// A generic [`Tool`] that adapts one manifest-declared external service.
pub struct ExternalServiceTool {
    entry: SkillManifestEntry,
}

impl ExternalServiceTool {
    pub fn new(entry: SkillManifestEntry) -> Self {
        Self { entry }
    }
}

#[async_trait]
impl Tool for ExternalServiceTool {
    fn name(&self) -> &str {
        &self.entry.name
    }
    fn description(&self) -> &str {
        &self.entry.description
    }
    fn required_tier(&self) -> u8 {
        self.entry.required_tier
    }
    fn is_write_operation(&self) -> bool {
        self.entry.write
    }

    async fn execute(
        &self,
        params: &str,
        _context: &ExecutionContext,
    ) -> Result<(String, TokenUsage), String> {
        let v: serde_json::Value = if params.trim().is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(params).map_err(|e| format!("invalid params: {e}"))?
        };

        let route_name = v
            .get("route")
            .and_then(|r| r.as_str())
            .ok_or("missing 'route' — call the 'skills' tool to list routes")?;
        let route_str =
            self.entry.routes.get(route_name).ok_or_else(|| {
                format!("skill '{}' has no route '{}'", self.entry.name, route_name)
            })?;
        let base = resolve_env(&self.entry.base_url).ok_or_else(|| {
            format!(
                "skill '{}' is not configured (set {})",
                self.entry.name, self.entry.base_url
            )
        })?;
        let (method, url) = build_invocation(route_str, &base, &v)?;

        let client = http();
        let mut req = match method.as_str() {
            "GET" => client.get(&url),
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "PATCH" => client.patch(&url),
            "DELETE" => client.delete(&url),
            other => return Err(format!("unsupported method '{other}'")),
        };
        if let (Some(header), Some(env_var)) = (&self.entry.auth_header, &self.entry.auth_env) {
            if let Ok(key) = std::env::var(env_var) {
                if !key.is_empty() {
                    req = req.header(header.as_str(), key);
                }
            }
        }
        if let Some(body) = v.get("body") {
            if !body.is_null() {
                req = req.json(body);
            }
        }

        let resp = req
            .send()
            .await
            .map_err(|e| format!("skill '{}' request failed: {e}", self.entry.name))?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(format!(
                "skill '{}' returned {status}: {text}",
                self.entry.name
            ));
        }
        Ok((text, TokenUsage::default()))
    }
}

/// Discovery tool (`skills`): lists every registered service skill and its routes.
pub struct SkillsListTool {
    skills: Arc<Mutex<Vec<SkillManifestEntry>>>,
}

impl SkillsListTool {
    pub fn new(skills: Arc<Mutex<Vec<SkillManifestEntry>>>) -> Self {
        Self { skills }
    }
}

#[async_trait]
impl Tool for SkillsListTool {
    fn name(&self) -> &str {
        "skills"
    }
    fn description(&self) -> &str {
        "List external service skills and their routes. Invoke one with \
         act <skill> {\"route\":\"<name>\",\"path\":{..},\"query\":{..},\"body\":{..}}"
    }
    fn required_tier(&self) -> u8 {
        0
    }
    fn is_write_operation(&self) -> bool {
        false
    }

    async fn execute(
        &self,
        _params: &str,
        _context: &ExecutionContext,
    ) -> Result<(String, TokenUsage), String> {
        let guard = self.skills.lock().map_err(|_| "skills registry poisoned")?;
        let arr: Vec<serde_json::Value> = guard
            .iter()
            .map(|e| {
                let mut routes: Vec<&String> = e.routes.keys().collect();
                routes.sort();
                serde_json::json!({
                    "name": e.name,
                    "description": e.description,
                    "required_tier": e.required_tier,
                    "write": e.write,
                    "configured": resolve_env(&e.base_url).is_some(),
                    "routes": routes,
                })
            })
            .collect();
        Ok((
            serde_json::to_string(&arr).unwrap_or_else(|_| "[]".to_string()),
            TokenUsage::default(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_manifest_has_vantage() {
        let m = default_manifest();
        let v = m.skills.iter().find(|s| s.name == "vantage").unwrap();
        assert_eq!(v.auth_header.as_deref(), Some("X-Agent-Key"));
        assert!(v.routes.contains_key("block_agents"));
    }

    #[test]
    fn resolve_env_passthrough_and_missing() {
        assert_eq!(
            resolve_env("http://host/x"),
            Some("http://host/x".to_string())
        );
        // A var that will not exist → None (skill not configured).
        assert_eq!(resolve_env("${OMOKODA_DEFINITELY_UNSET_VAR}/x"), None);
    }

    #[test]
    fn build_invocation_substitutes_path_and_query() {
        let params = serde_json::json!({
            "path": {"block_id": "default"},
            "query": {"capabilities": "1"}
        });
        let (method, url) = build_invocation(
            "GET /api/mesh/blocks/{block_id}/agents",
            "http://vantage:8080",
            &params,
        )
        .unwrap();
        assert_eq!(method, "GET");
        assert_eq!(
            url,
            "http://vantage:8080/api/mesh/blocks/default/agents?capabilities=1"
        );
    }

    #[test]
    fn build_invocation_rejects_unfilled_param() {
        let err =
            build_invocation("GET /x/{missing}", "http://h", &serde_json::json!({})).unwrap_err();
        assert!(err.contains("unfilled"));
    }

    #[test]
    fn manifest_json_round_trips() {
        let json = serde_json::to_string(&default_manifest()).unwrap();
        let back: SkillManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.skills.len(), 1);
        assert_eq!(back.skills[0].name, "vantage");
    }
}
