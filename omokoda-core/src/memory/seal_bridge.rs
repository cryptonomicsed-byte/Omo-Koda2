//! Sui Seal bridge — fetches the real decryption key (DEK) that gates
//! `memory/tee.rs`'s envelope from Seal's decentralized key servers,
//! instead of a static `NAUTILUS_SEAL_KEY` env-var secret.
//!
//! ## Why CLI-shellout, not the `seal-sdk-rs` crate
//!
//! `seal-sdk-rs` is the real, documented Rust SDK for Seal, but it pulls
//! Mysten's full monorepo transitively (`sui`, `fastcrypto`, `seal`,
//! `sui-rust-sdk`, `anemo`, ...) and hard-pins `chrono = "=0.4.39"`,
//! which conflicts with this workspace's `chrono 0.4.45` -- confirmed by
//! actually attempting the dependency (`cargo check` failed to resolve).
//! `onchain.rs` already made this exact call for the `sui` CLI over the
//! full Sui Rust SDK; this module follows the same precedent for Seal:
//! shell out to `seal-cli` (the real, documented Seal command-line tool)
//! rather than fight the dependency graph.
//!
//! ## `seal-cli fetch-keys`'s real signature (verified against source)
//!
//! Fetched Mysten's actual `crates/seal-cli/src/main.rs` (the docs page
//! for this 403'd) to get this right rather than guess. The real
//! `FetchKeys` subcommand is:
//!
//!   seal-cli fetch-keys --request <HEX> -k <ids> -t <threshold> -n <network> [--rpc-url <url>]
//!
//! Critically, `fetch-keys` takes exactly one identity input: `--request`,
//! a hex-encoded **BCS-serialized `FetchKeyRequest`** -- not a package id
//! or raw identity string. Building that request requires a signed
//! session-key certificate (the SDK's `SessionKey::new(package_id,
//! ttl_min, &mut wallet)` step from `seal-sdk-rs`'s own README example),
//! which the bare CLI has no subcommand to produce standalone. So a full
//! integration is two real, separate steps, both operator-configured
//! rather than fabricated here:
//!
//!   1. `SEAL_REQUEST_CMD` -- builds + signs the `FetchKeyRequest` and
//!      prints its hex encoding. This is real signing work this module
//!      does not attempt to reimplement (it needs a wallet key and the
//!      SDK's session-key logic) -- point it at whatever real signer the
//!      deployment has (a small `seal-sdk-rs`-based helper binary built
//!      outside this workspace's dependency graph, the TS SDK, etc.).
//!   2. `SEAL_FETCH_CMD` -- the verified `seal-cli fetch-keys` invocation
//!      above, with `{request_hex}` substituted from step 1's output.
//!
//! ## Wire format
//!
//! Whatever bytes `SEAL_FETCH_CMD` writes to stdout (a real Seal key
//! share response, or any format) are SHA-256'd down to a 32-byte DEK.
//! This is sound regardless of exact output encoding: a fixed output
//! length from arbitrary real input is exactly what a hash is for.
//!
//! Configuration (fail-open — unset means "Seal not wired", never a crash):
//!   SEAL_REQUEST_CMD     shell command that prints a hex-encoded, signed
//!                        FetchKeyRequest to stdout (see above)
//!   SEAL_FETCH_CMD       shell command template, e.g.:
//!                        "seal-cli fetch-keys --request {request_hex} \
//!                         -k {key_server_id} -t {threshold} -n {network}"
//!   SEAL_KEY_SERVER_IDS  comma-separated key server object ids
//!   SEAL_THRESHOLD       e.g. "2" for 2-of-3
//!   SEAL_NETWORK         "testnet" | "mainnet"

use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct SealConfig {
    pub request_cmd: String,
    pub fetch_cmd_template: String,
    pub key_server_ids: Vec<String>,
    pub threshold: String,
    pub network: String,
}

impl SealConfig {
    /// `None` = Seal not configured on this runtime; the caller should
    /// fall back to `TeeSealer::from_env`'s static key, not fail.
    pub fn from_env() -> Option<Self> {
        let request_cmd = std::env::var("SEAL_REQUEST_CMD").ok()?;
        let fetch_cmd_template = std::env::var("SEAL_FETCH_CMD").ok()?;
        let key_server_ids: Vec<String> = std::env::var("SEAL_KEY_SERVER_IDS")
            .ok()?
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if key_server_ids.is_empty() {
            return None;
        }
        let threshold = std::env::var("SEAL_THRESHOLD").unwrap_or_else(|_| "1".to_string());
        let network = std::env::var("SEAL_NETWORK").unwrap_or_else(|_| "testnet".to_string());
        Some(Self {
            request_cmd,
            fetch_cmd_template,
            key_server_ids,
            threshold,
            network,
        })
    }
}

/// Substitute the verified `fetch-keys` placeholders in `template` with
/// this fetch's real values. Pure — testable without ever shelling out.
fn build_fetch_command(config: &SealConfig, request_hex: &str) -> String {
    config
        .fetch_cmd_template
        .replace("{request_hex}", request_hex)
        .replace("{key_server_id}", &config.key_server_ids.join(","))
        .replace("{threshold}", &config.threshold)
        .replace("{network}", &config.network)
}

pub struct SealBridge {
    config: SealConfig,
}

impl SealBridge {
    pub fn new(config: SealConfig) -> Self {
        Self { config }
    }

    pub fn from_env() -> Option<Self> {
        SealConfig::from_env().map(Self::new)
    }

    /// Run a configured shell command and return its trimmed stdout as a
    /// UTF-8 string, or an error on nonzero exit / empty output. Shared by
    /// both real steps below -- the request-building step and the
    /// verified `fetch-keys` step have identical success/failure shape.
    async fn run(command: &str, label: &str) -> Result<Vec<u8>, String> {
        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .await
            .map_err(|e| format!("seal {label} failed to spawn: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "seal {label} command exited non-zero: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        if output.stdout.is_empty() {
            return Err(format!("seal {label} command produced no output"));
        }
        Ok(output.stdout)
    }

    /// Fetch this agent's DEK from Seal's key servers, gated by
    /// `omokoda::garden::seal_approve_agent_memory` on-chain (see
    /// garden.move). Two real steps, per the verified `seal-cli`
    /// signature documented at module level: (1) `SEAL_REQUEST_CMD`
    /// builds and signs the `FetchKeyRequest`, printing its hex encoding;
    /// (2) that hex is substituted into the verified `seal-cli fetch-keys
    /// --request {request_hex} -k ... -t ... -n ...` invocation. Whatever
    /// step 2 prints is SHA-256'd into the DEK.
    pub async fn fetch_dek(&self) -> Result<[u8; 32], String> {
        let request_bytes = Self::run(&self.config.request_cmd, "request-build").await?;
        let request_hex = String::from_utf8_lossy(&request_bytes).trim().to_string();

        let fetch_command = build_fetch_command(&self.config, &request_hex);
        let fetch_output = Self::run(&fetch_command, "fetch-keys").await?;

        let mut hasher = Sha256::new();
        hasher.update(b"omokoda:seal_dek_v1");
        hasher.update(&fetch_output);
        Ok(hasher.finalize().into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> SealConfig {
        SealConfig {
            request_cmd: "printf %s deadbeef".to_string(),
            fetch_cmd_template: "seal-cli fetch-keys --request {request_hex} -k {key_server_id} -t {threshold} -n {network}".to_string(),
            key_server_ids: vec!["0xserver1".to_string(), "0xserver2".to_string()],
            threshold: "2".to_string(),
            network: "testnet".to_string(),
        }
    }

    #[test]
    fn build_fetch_command_substitutes_every_verified_placeholder() {
        let cmd = build_fetch_command(&config(), "deadbeef");
        assert!(cmd.contains("--request deadbeef"));
        assert!(cmd.contains("-k 0xserver1,0xserver2"));
        assert!(cmd.contains("-t 2"));
        assert!(cmd.contains("-n testnet"));
        assert!(!cmd.contains('{'), "no placeholder should survive substitution");
    }

    #[test]
    fn from_env_is_none_without_request_cmd() {
        std::env::remove_var("SEAL_REQUEST_CMD");
        assert!(SealConfig::from_env().is_none());
    }

    #[test]
    fn from_env_is_none_without_key_servers() {
        std::env::set_var("SEAL_REQUEST_CMD", "echo test");
        std::env::set_var("SEAL_FETCH_CMD", "echo test");
        std::env::remove_var("SEAL_KEY_SERVER_IDS");
        assert!(SealConfig::from_env().is_none());
        std::env::remove_var("SEAL_REQUEST_CMD");
        std::env::remove_var("SEAL_FETCH_CMD");
    }

    #[tokio::test]
    async fn fetch_dek_derives_a_deterministic_key_from_the_two_step_pipeline() {
        let mut cfg = config();
        // Real subprocesses for both steps -- no seal-cli needed for this
        // test, just proving the request→fetch→hash pipeline itself is
        // deterministic and sound regardless of what real commands print.
        cfg.request_cmd = "printf %s fake-request-hex".to_string();
        cfg.fetch_cmd_template = "printf %s fixed-key-share-bytes".to_string();
        let bridge = SealBridge::new(cfg);
        let a = bridge.fetch_dek().await.unwrap();
        let b = bridge.fetch_dek().await.unwrap();
        assert_eq!(a, b, "same command output must derive the same DEK");
    }

    #[tokio::test]
    async fn fetch_dek_fails_when_request_step_produces_no_output() {
        let mut cfg = config();
        cfg.request_cmd = "true".to_string();
        let bridge = SealBridge::new(cfg);
        assert!(bridge.fetch_dek().await.is_err());
    }

    #[tokio::test]
    async fn fetch_dek_fails_when_fetch_step_exits_nonzero() {
        let mut cfg = config();
        cfg.fetch_cmd_template = "exit 1".to_string();
        let bridge = SealBridge::new(cfg);
        assert!(bridge.fetch_dek().await.is_err());
    }

    #[tokio::test]
    async fn fetch_dek_passes_the_real_request_hex_into_the_fetch_step() {
        let mut cfg = config();
        cfg.request_cmd = "printf %s cafef00d".to_string();
        // {request_hex} is substituted before the shell ever sees the
        // command, so this only succeeds if step 1's real output reached
        // step 2 -- proves the two steps are actually wired together, not
        // just independently working in isolation.
        cfg.fetch_cmd_template =
            "test \"{request_hex}\" = \"cafef00d\" && echo ok || exit 1".to_string();
        let bridge = SealBridge::new(cfg);
        assert!(bridge.fetch_dek().await.is_ok());
    }
}
