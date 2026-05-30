//! HTTP client implementations for external repos.
//! Wired when external services are running. Falls back to stubs when unavailable.
//!
//! Each client hits the service's REST API and deserializes typed results.
//! Base URLs are read from environment variables:
//!   BIPON_URL     (default: http://localhost:7401)
//!   VANITY_URL    (default: http://localhost:7402)
//!   RITUAL_URL    (default: http://localhost:7403)
//!   IFASCRIPT_URL (default: http://localhost:7404)
//!   NEX_URL       (default: http://localhost:7405)

use super::*;

fn base_url(env_var: &str, default: &str) -> String {
    std::env::var(env_var).unwrap_or_else(|_| default.to_string())
}

// ─── HttpBiponClient ──────────────────────────────────────────────────────────

pub struct HttpBiponClient {
    base: String,
}

impl HttpBiponClient {
    pub fn new() -> Self {
        Self {
            base: base_url("BIPON_URL", "http://localhost:7401"),
        }
    }
}

impl Default for HttpBiponClient {
    fn default() -> Self {
        Self::new()
    }
}

impl BiponClient for HttpBiponClient {
    fn entropy_to_mnemonic(&self, _entropy: &[u8]) -> Result<MnemonicResult, ClientError> {
        Err(ClientError::Unavailable(format!(
            "{} — HTTP client not yet wired",
            self.base
        )))
    }

    fn mnemonic_to_seed(&self, _phrase: &str, _passphrase: &str) -> Result<[u8; 64], ClientError> {
        Err(ClientError::Unavailable(format!(
            "{} — HTTP client not yet wired",
            self.base
        )))
    }

    fn personality_profile(&self, _mnemonic: &str) -> Result<PersonalityResult, ClientError> {
        Err(ClientError::Unavailable(format!(
            "{} — HTTP client not yet wired",
            self.base
        )))
    }
}

// ─── HttpVanityClient ─────────────────────────────────────────────────────────

pub struct HttpVanityClient {
    base: String,
}

impl HttpVanityClient {
    pub fn new() -> Self {
        Self {
            base: base_url("VANITY_URL", "http://localhost:7402"),
        }
    }
}

impl Default for HttpVanityClient {
    fn default() -> Self {
        Self::new()
    }
}

impl VanityClient for HttpVanityClient {
    fn derive_wallet(
        &self,
        _mnemonic: &str,
        _passphrase: &str,
    ) -> Result<WalletResult, ClientError> {
        Err(ClientError::Unavailable(format!(
            "{} — HTTP client not yet wired",
            self.base
        )))
    }

    fn cloak_display(&self, _words: &[String], _offset: u8) -> Result<CloakResult, ClientError> {
        Err(ClientError::Unavailable(format!(
            "{} — HTTP client not yet wired",
            self.base
        )))
    }

    fn scan_poison(
        &self,
        _candidate: &str,
        _known: &[String],
    ) -> Result<PoisonScanResult, ClientError> {
        Err(ClientError::Unavailable(format!(
            "{} — HTTP client not yet wired",
            self.base
        )))
    }
}

// ─── HttpRitualClient ─────────────────────────────────────────────────────────

pub struct HttpRitualClient {
    base: String,
}

impl HttpRitualClient {
    pub fn new() -> Self {
        Self {
            base: base_url("RITUAL_URL", "http://localhost:7403"),
        }
    }
}

impl Default for HttpRitualClient {
    fn default() -> Self {
        Self::new()
    }
}

impl RitualClient for HttpRitualClient {
    fn verify_pocw(&self, _tier: u8, _steps: u64) -> Result<PocwResult, ClientError> {
        Err(ClientError::Unavailable(format!(
            "{} — HTTP client not yet wired",
            self.base
        )))
    }

    fn score_bbu(&self, _code: &str) -> Result<BbuResult, ClientError> {
        Err(ClientError::Unavailable(format!(
            "{} — HTTP client not yet wired",
            self.base
        )))
    }

    fn augury_predict(&self, _patterns: &[MemoryPattern]) -> Result<AuguryResult, ClientError> {
        Err(ClientError::Unavailable(format!(
            "{} — HTTP client not yet wired",
            self.base
        )))
    }
}

// ─── HttpIfascriptClient ──────────────────────────────────────────────────────

pub struct HttpIfascriptClient {
    base: String,
}

impl HttpIfascriptClient {
    pub fn new() -> Self {
        Self {
            base: base_url("IFASCRIPT_URL", "http://localhost:7404"),
        }
    }
}

impl Default for HttpIfascriptClient {
    fn default() -> Self {
        Self::new()
    }
}

impl IfascriptClient for HttpIfascriptClient {
    fn lookup_odu(&self, _index: u8) -> Result<OduResult, ClientError> {
        Err(ClientError::Unavailable(format!(
            "{} — HTTP client not yet wired",
            self.base
        )))
    }

    fn cast_ebo(&self, _odu: u8) -> Result<EboResult, ClientError> {
        Err(ClientError::Unavailable(format!(
            "{} — HTTP client not yet wired",
            self.base
        )))
    }

    fn generate_entropy(&self, _seed: &[u8]) -> Result<Vec<u8>, ClientError> {
        Err(ClientError::Unavailable(format!(
            "{} — HTTP client not yet wired",
            self.base
        )))
    }

    fn larql_query(&self, _query: &str, _tier: u8) -> Result<LarqlResult, ClientError> {
        Err(ClientError::Unavailable(format!(
            "{} — HTTP client not yet wired",
            self.base
        )))
    }
}

// ─── HttpNexClient ────────────────────────────────────────────────────────────

pub struct HttpNexClient {
    base: String,
}

impl HttpNexClient {
    pub fn new() -> Self {
        Self {
            base: base_url("NEX_URL", "http://localhost:7405"),
        }
    }
}

impl Default for HttpNexClient {
    fn default() -> Self {
        Self::new()
    }
}

impl NexClient for HttpNexClient {
    fn submit_graph(&self, _nodes: Vec<GraphNode>) -> Result<GraphResult, ClientError> {
        Err(ClientError::Unavailable(format!(
            "{} — HTTP client not yet wired",
            self.base
        )))
    }

    fn graph_status(&self, _graph_id: &str) -> Result<GraphStatus, ClientError> {
        Err(ClientError::Unavailable(format!(
            "{} — HTTP client not yet wired",
            self.base
        )))
    }
}
