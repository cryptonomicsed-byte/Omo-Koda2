/// OAuth 2.0 + PKCE credential flow for third-party service access.
/// Agents obtain authenticated access to GitHub, Google, etc. without storing raw secrets.
///
/// Security invariants:
/// - Code verifier is zeroized on drop.
/// - Tokens are stored in the `vault` module, not in plaintext.
/// - PKCE prevents authorization-code interception attacks.
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use zeroize::Zeroize;

// ---------------------------------------------------------------------------
// PKCE helpers
// ---------------------------------------------------------------------------

/// A PKCE code verifier — 32 random bytes base64url-encoded (no padding).
/// The struct zeroizes its memory on drop.
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct CodeVerifier(String);

impl CodeVerifier {
    /// Generate a fresh code verifier using the OS CSPRNG.
    #[must_use]
    pub fn generate() -> Self {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        Self(base64url_encode(&bytes))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Derive the PKCE code challenge (S256 method).
    #[must_use]
    pub fn challenge(&self) -> String {
        let hash = Sha256::digest(self.0.as_bytes());
        base64url_encode(&hash)
    }
}

impl std::fmt::Debug for CodeVerifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CodeVerifier([redacted])")
    }
}

fn base64url_encode(bytes: &[u8]) -> String {
    // Base64url without padding — matches RFC 7636
    let b64 = base64_encode(bytes);
    b64.trim_end_matches('=')
        .replace('+', "-")
        .replace('/', "_")
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = if chunk.len() > 1 { chunk[1] as usize } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as usize } else { 0 };
        out.push(TABLE[(b0 >> 2) & 0x3F] as char);
        out.push(TABLE[((b0 << 4) | (b1 >> 4)) & 0x3F] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((b1 << 2) | (b2 >> 6)) & 0x3F] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[b2 & 0x3F] as char);
        } else {
            out.push('=');
        }
    }
    out
}

// ---------------------------------------------------------------------------
// OAuth provider configuration
// ---------------------------------------------------------------------------

/// Well-known OAuth 2.0 providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OAuthProvider {
    GitHub,
    Google,
    Linear,
    Slack,
    Custom,
}

impl OAuthProvider {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::GitHub => "github",
            Self::Google => "google",
            Self::Linear => "linear",
            Self::Slack => "slack",
            Self::Custom => "custom",
        }
    }
}

/// Static configuration for one OAuth provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub provider: OAuthProvider,
    pub client_id: String,
    pub authorize_url: String,
    pub token_url: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

impl OAuthConfig {
    /// Build the authorization URL for the PKCE flow.
    /// The caller must persist `verifier` until the callback is received.
    #[must_use]
    pub fn authorization_url(&self, verifier: &CodeVerifier, state: &str) -> String {
        let scopes_joined = self.scopes.join(" ");
        let scope = urlencoding::encode(&scopes_joined);
        let challenge = verifier.challenge();
        format!(
            "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
            self.authorize_url,
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.redirect_uri),
            scope,
            urlencoding::encode(state),
            urlencoding::encode(&challenge),
        )
    }
}

// ---------------------------------------------------------------------------
// Token types
// ---------------------------------------------------------------------------

/// A token set returned after a successful authorization code exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSet {
    pub access_token: String,
    pub refresh_token: Option<String>,
    /// Expiry as Unix timestamp (seconds). `None` = never expires.
    pub expires_at: Option<u64>,
    pub token_type: String,
    pub scope: Option<String>,
}

impl TokenSet {
    /// True if the access token has expired (or will expire within `buffer_secs`).
    #[must_use]
    pub fn is_expired(&self, buffer_secs: u64) -> bool {
        match self.expires_at {
            None => false,
            Some(exp) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                now + buffer_secs >= exp
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Token exchange request/response (for test stubs and real HTTP)
// ---------------------------------------------------------------------------

/// Parameters for the token endpoint POST.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub code: String,
    pub redirect_uri: String,
    pub client_id: String,
    pub code_verifier: String,
}

impl TokenRequest {
    /// Build an authorization_code exchange request.
    pub fn authorization_code(
        code: &str,
        verifier: &CodeVerifier,
        config: &OAuthConfig,
    ) -> Self {
        Self {
            grant_type: "authorization_code".to_string(),
            code: code.to_string(),
            redirect_uri: config.redirect_uri.clone(),
            client_id: config.client_id.clone(),
            code_verifier: verifier.as_str().to_string(),
        }
    }
}

/// Refresh token request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshRequest {
    pub grant_type: String,
    pub refresh_token: String,
    pub client_id: String,
}

impl RefreshRequest {
    pub fn new(refresh_token: &str, client_id: &str) -> Self {
        Self {
            grant_type: "refresh_token".to_string(),
            refresh_token: refresh_token.to_string(),
            client_id: client_id.to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// In-memory credential store (swap in vault for production)
// ---------------------------------------------------------------------------

/// Stores token sets keyed by provider name.
/// In production this delegates to `identity::vault` for encrypted persistence.
#[derive(Debug, Default)]
pub struct CredentialStore {
    tokens: HashMap<String, TokenSet>,
}

impl CredentialStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn store(&mut self, provider: OAuthProvider, token_set: TokenSet) {
        self.tokens.insert(provider.name().to_string(), token_set);
    }

    #[must_use]
    pub fn get(&self, provider: OAuthProvider) -> Option<&TokenSet> {
        self.tokens.get(provider.name())
    }

    /// Returns a valid (non-expired) access token, or `None` if missing / expired.
    /// Callers should trigger a refresh flow when this returns `None`.
    #[must_use]
    pub fn valid_token(&self, provider: OAuthProvider) -> Option<&str> {
        self.get(provider)
            .filter(|t| !t.is_expired(60))
            .map(|t| t.access_token.as_str())
    }

    pub fn revoke(&mut self, provider: OAuthProvider) {
        self.tokens.remove(provider.name());
    }
}

// ---------------------------------------------------------------------------
// State parameter helpers
// ---------------------------------------------------------------------------

/// Generate a random CSRF state parameter for the OAuth redirect.
#[must_use]
pub fn generate_state() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn github_config() -> OAuthConfig {
        OAuthConfig {
            provider: OAuthProvider::GitHub,
            client_id: "test_client_id".to_string(),
            authorize_url: "https://github.com/login/oauth/authorize".to_string(),
            token_url: "https://github.com/login/oauth/access_token".to_string(),
            redirect_uri: "http://localhost:3000/callback".to_string(),
            scopes: vec!["repo".to_string(), "read:user".to_string()],
        }
    }

    #[test]
    fn code_verifier_generates_valid_challenge() {
        let verifier = CodeVerifier::generate();
        let challenge = verifier.challenge();
        // S256 challenge is base64url of SHA-256 of verifier
        assert!(!challenge.is_empty());
        // No padding or invalid chars
        assert!(!challenge.contains('='));
        assert!(!challenge.contains('+'));
        assert!(!challenge.contains('/'));
    }

    #[test]
    fn two_verifiers_are_different() {
        let v1 = CodeVerifier::generate();
        let v2 = CodeVerifier::generate();
        assert_ne!(v1.as_str(), v2.as_str());
    }

    #[test]
    fn authorization_url_contains_challenge_and_state() {
        let config = github_config();
        let verifier = CodeVerifier::generate();
        let state = "csrf_test_state";
        let url = config.authorization_url(&verifier, state);
        assert!(url.contains("code_challenge="));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("csrf_test_state"));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("test_client_id"));
    }

    #[test]
    fn token_set_not_expired_when_no_expiry() {
        let token = TokenSet {
            access_token: "tok".to_string(),
            refresh_token: None,
            expires_at: None,
            token_type: "Bearer".to_string(),
            scope: None,
        };
        assert!(!token.is_expired(0));
    }

    #[test]
    fn token_set_expired_in_past() {
        let token = TokenSet {
            access_token: "tok".to_string(),
            refresh_token: None,
            expires_at: Some(1),
            token_type: "Bearer".to_string(),
            scope: None,
        };
        assert!(token.is_expired(0));
    }

    #[test]
    fn credential_store_valid_token_returns_none_when_expired() {
        let mut store = CredentialStore::new();
        store.store(
            OAuthProvider::GitHub,
            TokenSet {
                access_token: "old".to_string(),
                refresh_token: None,
                expires_at: Some(1),
                token_type: "Bearer".to_string(),
                scope: None,
            },
        );
        assert!(store.valid_token(OAuthProvider::GitHub).is_none());
    }

    #[test]
    fn credential_store_valid_token_returns_token_when_fresh() {
        let mut store = CredentialStore::new();
        let future = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600;
        store.store(
            OAuthProvider::Google,
            TokenSet {
                access_token: "fresh_token".to_string(),
                refresh_token: None,
                expires_at: Some(future),
                token_type: "Bearer".to_string(),
                scope: Some("openid".to_string()),
            },
        );
        assert_eq!(
            store.valid_token(OAuthProvider::Google),
            Some("fresh_token")
        );
    }

    #[test]
    fn revoke_removes_token() {
        let mut store = CredentialStore::new();
        store.store(
            OAuthProvider::Slack,
            TokenSet {
                access_token: "tok".to_string(),
                refresh_token: None,
                expires_at: None,
                token_type: "Bearer".to_string(),
                scope: None,
            },
        );
        assert!(store.get(OAuthProvider::Slack).is_some());
        store.revoke(OAuthProvider::Slack);
        assert!(store.get(OAuthProvider::Slack).is_none());
    }

    #[test]
    fn token_request_authorization_code_sets_grant_type() {
        let config = github_config();
        let verifier = CodeVerifier::generate();
        let req = TokenRequest::authorization_code("auth_code_xyz", &verifier, &config);
        assert_eq!(req.grant_type, "authorization_code");
        assert_eq!(req.code, "auth_code_xyz");
        assert_eq!(req.client_id, "test_client_id");
    }

    #[test]
    fn generate_state_is_hex_and_unique() {
        let s1 = generate_state();
        let s2 = generate_state();
        assert_eq!(s1.len(), 32);
        assert!(s1.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(s1, s2);
    }

    #[test]
    fn provider_name_roundtrips() {
        assert_eq!(OAuthProvider::GitHub.name(), "github");
        assert_eq!(OAuthProvider::Google.name(), "google");
    }
}
