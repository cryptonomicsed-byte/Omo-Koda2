use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A scoped capability grant.
/// Capabilities complement tier-based permissions: they allow temporary,
/// constrained elevation for specific operations without changing the agent's tier.
///
/// Called FROM WITHIN `act` — the executor checks capabilities before dispatching.
/// Stored in the session envelope (encrypted) so grants survive think/act cycles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityToken {
    /// Stable identifier derived from scope + granted_by + issued_at
    pub id: String,
    /// e.g. "file:read", "file:write", "network:fetch", "system:exec"
    pub scope: String,
    /// Who granted this capability (policy name, user ID, or "system")
    pub granted_by: String,
    /// Unix timestamp when the token was issued
    pub issued_at: u64,
    /// Unix timestamp when the token expires (None = session-only, expires on SessionEnd)
    pub expires_at: Option<u64>,
    /// Structural constraints: ["path=/workspace/*", "host=api.github.com"]
    pub constraints: Vec<String>,
}

impl CapabilityToken {
    pub fn new(
        scope: impl Into<String>,
        granted_by: impl Into<String>,
        expires_at: Option<u64>,
        constraints: Vec<String>,
    ) -> Self {
        let scope = scope.into();
        let granted_by = granted_by.into();
        let issued_at = unix_now();
        let id = derive_id(&scope, &granted_by, issued_at);
        Self {
            id,
            scope,
            granted_by,
            issued_at,
            expires_at,
            constraints,
        }
    }

    /// Returns true if this token is still valid at the given Unix timestamp.
    #[must_use]
    pub fn is_valid_at(&self, now: u64) -> bool {
        match self.expires_at {
            Some(exp) => now < exp,
            None => true,
        }
    }

    /// Returns true if the token covers the requested action path.
    /// Constraints use simple glob-style `*` wildcards on the value.
    #[must_use]
    pub fn satisfies_constraint(&self, key: &str, value: &str) -> bool {
        if self.constraints.is_empty() {
            return true;
        }
        let prefix = format!("{key}=");
        // All constraints for this key must match; if no constraint for this key, pass
        let relevant: Vec<&str> = self
            .constraints
            .iter()
            .filter(|c| c.starts_with(&prefix))
            .map(|c| c.trim_start_matches(&prefix))
            .collect();

        if relevant.is_empty() {
            return true;
        }
        relevant.iter().any(|pattern| glob_match(pattern, value))
    }
}

/// Registry of active capability tokens for a session.
/// Stored as part of the session state; tokens are cleared on SessionEnd unless persistent.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilityRegistry {
    tokens: HashMap<String, CapabilityToken>,
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Grant a new capability token. Returns the token ID.
    pub fn grant(
        &mut self,
        scope: impl Into<String>,
        granted_by: impl Into<String>,
        expires_at: Option<u64>,
        constraints: Vec<String>,
    ) -> String {
        let token = CapabilityToken::new(scope, granted_by, expires_at, constraints);
        let id = token.id.clone();
        self.tokens.insert(id.clone(), token);
        id
    }

    /// Revoke a capability by ID. Returns true if the token existed.
    pub fn revoke(&mut self, token_id: &str) -> bool {
        self.tokens.remove(token_id).is_some()
    }

    /// Check if the registry holds a valid, non-expired token for the given scope.
    /// Optionally check a constraint key/value pair (e.g. "path", "/workspace/main.rs").
    #[must_use]
    pub fn has_capability(
        &self,
        scope: &str,
        constraint_key: Option<&str>,
        constraint_value: Option<&str>,
    ) -> bool {
        let now = unix_now();
        self.tokens.values().any(|t| {
            t.scope == scope
                && t.is_valid_at(now)
                && match (constraint_key, constraint_value) {
                    (Some(k), Some(v)) => t.satisfies_constraint(k, v),
                    _ => true,
                }
        })
    }

    /// Purge all expired tokens. Called at session boundaries.
    pub fn purge_expired(&mut self) {
        let now = unix_now();
        self.tokens.retain(|_, t| t.is_valid_at(now));
    }

    /// All active token IDs and their scopes.
    #[must_use]
    pub fn active_grants(&self) -> Vec<(String, String)> {
        let now = unix_now();
        self.tokens
            .values()
            .filter(|t| t.is_valid_at(now))
            .map(|t| (t.id.clone(), t.scope.clone()))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}

// ─── Internal helpers ────────────────────────────────────────────────────────

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn derive_id(scope: &str, granted_by: &str, issued_at: u64) -> String {
    // Simple deterministic ID: first 16 hex chars of a hash-like mix
    let raw = format!("{scope}:{granted_by}:{issued_at}");
    let mut hash: u64 = 0xcbf29ce484222325; // FNV-1a basis
    for byte in raw.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", hash)
}

/// Minimal glob match: `*` matches any substring, no other special chars.
fn glob_match(pattern: &str, text: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if !pattern.contains('*') {
        return pattern == text;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    let mut pos = 0usize;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 {
            if !text.starts_with(part) {
                return false;
            }
            pos = part.len();
        } else if i == parts.len() - 1 {
            return text[pos..].ends_with(part);
        } else if let Some(idx) = text[pos..].find(part) {
            pos += idx + part.len();
        } else {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grant_and_check_capability() {
        let mut registry = CapabilityRegistry::new();
        registry.grant("file:read", "system", None, vec![]);
        assert!(registry.has_capability("file:read", None, None));
        assert!(!registry.has_capability("file:write", None, None));
    }

    #[test]
    fn revoke_removes_capability() {
        let mut registry = CapabilityRegistry::new();
        let id = registry.grant("network:fetch", "user", None, vec![]);
        assert!(registry.has_capability("network:fetch", None, None));
        assert!(registry.revoke(&id));
        assert!(!registry.has_capability("network:fetch", None, None));
    }

    #[test]
    fn expired_token_not_valid() {
        let token = CapabilityToken {
            id: "test".to_string(),
            scope: "file:read".to_string(),
            granted_by: "system".to_string(),
            issued_at: 1000,
            expires_at: Some(1001), // already expired
            constraints: vec![],
        };
        assert!(!token.is_valid_at(5000));
        assert!(token.is_valid_at(1000));
    }

    #[test]
    fn constraint_path_glob_matches() {
        let token = CapabilityToken::new(
            "file:read",
            "system",
            None,
            vec!["path=/workspace/*".to_string()],
        );
        assert!(token.satisfies_constraint("path", "/workspace/main.rs"));
        assert!(!token.satisfies_constraint("path", "/etc/passwd"));
    }

    #[test]
    fn constraint_exact_host_match() {
        let token = CapabilityToken::new(
            "network:fetch",
            "user",
            None,
            vec!["host=api.github.com".to_string()],
        );
        assert!(token.satisfies_constraint("host", "api.github.com"));
        assert!(!token.satisfies_constraint("host", "evil.com"));
    }

    #[test]
    fn no_constraints_always_satisfies() {
        let token = CapabilityToken::new("file:read", "system", None, vec![]);
        assert!(token.satisfies_constraint("path", "/anywhere"));
    }

    #[test]
    fn purge_expired_cleans_registry() {
        let mut registry = CapabilityRegistry::new();
        // Add an already-expired token directly
        let mut expired = CapabilityToken::new("file:read", "system", Some(1), vec![]);
        expired.issued_at = 0;
        registry.tokens.insert(expired.id.clone(), expired);
        // Add a valid token
        registry.grant("network:fetch", "user", None, vec![]);
        assert_eq!(registry.len(), 2);
        registry.purge_expired();
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn active_grants_lists_valid_tokens() {
        let mut registry = CapabilityRegistry::new();
        registry.grant("file:read", "system", None, vec![]);
        registry.grant("file:write", "user", None, vec![]);
        let grants = registry.active_grants();
        assert_eq!(grants.len(), 2);
        let scopes: Vec<&str> = grants.iter().map(|(_, s)| s.as_str()).collect();
        assert!(scopes.contains(&"file:read"));
        assert!(scopes.contains(&"file:write"));
    }

    #[test]
    fn glob_match_wildcard_prefix() {
        assert!(glob_match("*.rs", "main.rs"));
        assert!(!glob_match("*.rs", "main.ts"));
        assert!(glob_match("/workspace/*", "/workspace/src/lib.rs"));
    }
}
