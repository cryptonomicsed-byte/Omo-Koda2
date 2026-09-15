//! `walletd` — runtime daemon for [`AgentComputeWallet`].
//!
//! Responsibilities (AGENT_COMPUTE_WALLET_SPEC.md):
//! 1. Apply lazy decay on every read/write via `apply_decay()` (already in wallet.rs).
//! 2. Maintain `ComputeLedgerEntry` log (already in wallet.rs).
//! 3. Enforce stake locks — available = balance − staked; prevent overspend.
//! 4. Submit `VerifiedGPUWork` to OSOVM for Dopamine credit (POST /run GPU_CONTRIBUTION).
//! 5. Request Synapse conversion via OSOVM TOC_MINT (POST /run TOC_MINT).
//! 6. Sync wallet state to Vantage on every heartbeat tick (PUT /api/agents/{id}/wallet).
//! 7. Alert agent runtime when Synapse < LOW_WATER_MARK (emit [`ComputeWarning`]).
//!
//! # Lifecycle
//! ```rust,ignore
//! let walletd = Arc::new(Walletd::new(
//!     AgentComputeWallet::birth_endowment("agent:42"),
//!     WalletdConfig { agent_id: "42".into(), ..Default::default() },
//! ));
//! tokio::spawn(walletd.clone().run());
//! ```

use crate::kernel::compute::wallet::{AgentComputeWallet, LOW_WATER_MARK};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

// ── Configuration ─────────────────────────────────────────────────────────────

/// Runtime configuration for a [`Walletd`] instance.
#[derive(Debug, Clone)]
pub struct WalletdConfig {
    /// The agent's stable identifier (used for OSOVM calls and the Vantage URL path).
    pub agent_id: String,
    /// Base URL of the OSOVM simulation/token engine (default: `http://127.0.0.1:7780`).
    pub osovm_url: String,
    /// Base URL of the Vantage social/hub backend (default: `http://127.0.0.1:7700`).
    pub vantage_url: String,
    /// How often the daemon loop ticks in seconds (default: 60).
    pub sync_interval_secs: u64,
}

impl Default for WalletdConfig {
    fn default() -> Self {
        Self {
            agent_id: String::new(),
            osovm_url: std::env::var("OSOVM_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:7780".into()),
            vantage_url: std::env::var("VANTAGE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:7700".into()),
            sync_interval_secs: 60,
        }
    }
}

// ── Daemon ────────────────────────────────────────────────────────────────────

/// Live wallet daemon. Wrap in `Arc` and spawn with `tokio::spawn(daemon.clone().run())`.
pub struct Walletd {
    /// Shared, mutex-guarded wallet state. Safe to read/write from multiple tasks.
    pub wallet: Arc<Mutex<AgentComputeWallet>>,
    config: WalletdConfig,
    client: reqwest::Client,
}

impl Walletd {
    /// Create a new daemon wrapping `wallet` with the given `config`.
    pub fn new(wallet: AgentComputeWallet, config: WalletdConfig) -> Self {
        Self {
            wallet: Arc::new(Mutex::new(wallet)),
            config,
            client: reqwest::Client::new(),
        }
    }

    // ── Reads ────────────────────────────────────────────────────────────────

    /// Return a snapshot of the wallet with lazy decay already applied.
    /// Always call this instead of locking the mutex directly for reads.
    pub async fn read(&self) -> AgentComputeWallet {
        let mut w = self.wallet.lock().await;
        w.apply_decay();
        w.clone()
    }

    // ── Writes ───────────────────────────────────────────────────────────────

    /// Spend `amount` Synapse for `reason`.
    ///
    /// Enforces stake locks: available = `synapse_balance − synapse_staked`.
    /// Any active lock amounts are excluded from the spendable pool.
    pub async fn spend_synapse(&self, amount: u64, reason: &str) -> Result<(), String> {
        let mut w = self.wallet.lock().await;
        w.apply_decay();

        // Only the unstaked portion is spendable.
        let available = w.synapse_balance.saturating_sub(w.synapse_staked);
        if available < amount {
            return Err(format!(
                "insufficient available synapse: {} available \
                 ({} balance − {} staked), need {}",
                available, w.synapse_balance, w.synapse_staked, amount
            ));
        }

        // Delegate to the wallet's own spend logic (which also appends a ledger entry).
        w.spend_synapse(amount, reason)
    }

    // ── OSOVM calls ──────────────────────────────────────────────────────────

    /// Submit `VerifiedGPUWork` to OSOVM and credit the earned Dopamine.
    ///
    /// OSOVM opcode: `GPU_CONTRIBUTION`.
    /// On success returns the Dopamine amount credited.
    pub async fn submit_gpu_work(
        &self,
        gpu_seconds: f64,
        hardware_attestation: &str,
        workload_hash: &str,
    ) -> Result<u64, String> {
        let resp = self
            .client
            .post(format!("{}/run", self.config.osovm_url))
            .json(&serde_json::json!({
                "opcode": "GPU_CONTRIBUTION",
                "args": {
                    "agent_id":             self.config.agent_id,
                    "gpu_seconds":          gpu_seconds,
                    "hardware_attestation": hardware_attestation,
                    "workload_hash":        workload_hash,
                }
            }))
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| format!("GPU_CONTRIBUTION request failed: {e}"))?;

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("GPU_CONTRIBUTION response parse failed: {e}"))?;

        if body["success"].as_bool().unwrap_or(false) {
            let earned = body["dopamine_earned"].as_u64().unwrap_or(0);
            let receipt_id = body["event_id"].as_str().map(String::from);
            {
                let mut w = self.wallet.lock().await;
                w.credit_dopamine(earned, receipt_id);
            }
            tracing::info!(
                agent_id = %self.config.agent_id,
                gpu_seconds,
                dopamine_earned = earned,
                "GPU_CONTRIBUTION credited"
            );
            Ok(earned)
        } else {
            let msg = body["error"]
                .as_str()
                .unwrap_or("OSOVM GPU_CONTRIBUTION failed")
                .to_string();
            Err(msg)
        }
    }

    /// Convert `synapse_amount` Synapse via the OSOVM TOC_MINT opcode.
    ///
    /// OSOVM validates the 10:1 burn ratio server-side; on success the local
    /// wallet also executes `convert_dopamine_to_synapse` to stay in sync.
    pub async fn request_synapse_conversion(&self, synapse_amount: u64) -> Result<u64, String> {
        // gpu_seconds is a proxy arg accepted by the TOC_MINT opcode — derive
        // from synapse_amount * 10 (dopamine burned) normalised to seconds.
        let gpu_seconds = (synapse_amount as f64 * 10.0) / 1_000_000.0;

        let resp = self
            .client
            .post(format!("{}/run", self.config.osovm_url))
            .json(&serde_json::json!({
                "opcode": "TOC_MINT",
                "args": {
                    "agent_id":   self.config.agent_id,
                    "gpu_seconds": gpu_seconds,
                }
            }))
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| format!("TOC_MINT request failed: {e}"))?;

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("TOC_MINT response parse failed: {e}"))?;

        if body["success"].as_bool().unwrap_or(false) {
            let mut w = self.wallet.lock().await;
            w.convert_dopamine_to_synapse(synapse_amount)?;
            tracing::info!(
                agent_id = %self.config.agent_id,
                synapse_amount,
                "TOC_MINT: Dopamine→Synapse conversion committed"
            );
            Ok(synapse_amount)
        } else {
            Err(body["error"]
                .as_str()
                .unwrap_or("TOC_MINT failed")
                .to_string())
        }
    }

    // ── Vantage sync ─────────────────────────────────────────────────────────

    /// Push a wallet snapshot to Vantage. Fire-and-forget — errors are logged
    /// but never propagate (Vantage being down must not crash the local agent).
    pub async fn sync_to_vantage(&self) {
        let snapshot = self.read().await;
        let url = format!(
            "{}/api/agents/{}/wallet",
            self.config.vantage_url, self.config.agent_id
        );
        match self
            .client
            .put(&url)
            .json(&snapshot)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                tracing::debug!(
                    agent_id = %self.config.agent_id,
                    "wallet synced to Vantage"
                );
            }
            Ok(resp) => {
                tracing::warn!(
                    agent_id = %self.config.agent_id,
                    status = %resp.status(),
                    "Vantage wallet sync returned non-2xx"
                );
            }
            Err(e) => {
                tracing::warn!(
                    agent_id = %self.config.agent_id,
                    error = %e,
                    "Vantage wallet sync failed (fire-and-forget)"
                );
            }
        }
    }

    // ── Low-water check ──────────────────────────────────────────────────────

    /// Return a [`ComputeWarning`] when Synapse is below [`LOW_WATER_MARK`],
    /// or `None` when the balance is healthy.
    pub async fn check_low_water(&self) -> Option<ComputeWarning> {
        let w = self.read().await;
        if w.is_low_water() {
            let severity = if w.synapse_balance == 0 {
                WarningSeverity::Critical
            } else {
                WarningSeverity::Warning
            };
            Some(ComputeWarning {
                agent_id: self.config.agent_id.clone(),
                synapse_balance: w.synapse_balance,
                low_water_mark: LOW_WATER_MARK,
                severity,
            })
        } else {
            None
        }
    }

    // ── Daemon loop ───────────────────────────────────────────────────────────

    /// Main daemon loop. Spawn with `tokio::spawn(walletd.clone().run())`.
    ///
    /// Each tick (default every 60 s):
    /// 1. Apply lazy decay to the wallet.
    /// 2. Sync the snapshot to Vantage.
    /// 3. Check the Synapse low-water mark and emit a `tracing::warn!` if hit.
    pub async fn run(self: Arc<Self>) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(
            self.config.sync_interval_secs,
        ));
        tracing::info!(
            agent_id = %self.config.agent_id,
            interval_secs = self.config.sync_interval_secs,
            "walletd started"
        );
        loop {
            interval.tick().await;

            // 1. Apply decay
            {
                let mut w = self.wallet.lock().await;
                w.apply_decay();
            }

            // 2. Sync to Vantage
            self.sync_to_vantage().await;

            // 3. Low-water alert
            if let Some(warning) = self.check_low_water().await {
                tracing::warn!(
                    agent_id  = %warning.agent_id,
                    balance   = warning.synapse_balance,
                    mark      = warning.low_water_mark,
                    severity  = ?warning.severity,
                    "ComputeWarning: Synapse low"
                );
            }
        }
    }
}

// ── Warning types ─────────────────────────────────────────────────────────────

/// Emitted by [`Walletd::check_low_water`] when the agent's Synapse balance
/// drops below [`LOW_WATER_MARK`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeWarning {
    pub agent_id: String,
    pub synapse_balance: u64,
    pub low_water_mark: u64,
    pub severity: WarningSeverity,
}

/// Severity classification for a [`ComputeWarning`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarningSeverity {
    /// Synapse is below `LOW_WATER_MARK` but above zero.
    Warning,
    /// Synapse is exactly zero — agent is compute-starved (COMPUTE_STARVED state).
    Critical,
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::compute::wallet::AgentComputeWallet;

    fn make_daemon(wallet: AgentComputeWallet) -> Walletd {
        Walletd::new(
            wallet,
            WalletdConfig {
                agent_id: "agent:test".into(),
                ..Default::default()
            },
        )
    }

    #[tokio::test]
    async fn read_applies_decay() {
        let w = AgentComputeWallet::birth_endowment("agent:test");
        let d = make_daemon(w);
        let snap = d.read().await;
        // last_decay_tick must be set (birth_endowment records now_secs)
        assert!(snap.last_decay_tick > 0);
    }

    #[tokio::test]
    async fn spend_synapse_respects_stake_lock() {
        let mut w = AgentComputeWallet::birth_endowment("agent:test");
        // Stake all but 5 Synapse
        let bal = w.synapse_balance;
        w.lock_stake("lock:1", "tier_gate", bal - 5, 0).unwrap();
        let d = make_daemon(w);

        // Spending 5 should succeed
        d.spend_synapse(5, "test").await.unwrap();

        // Spending 1 more should fail (nothing left)
        let err = d.spend_synapse(1, "test").await.unwrap_err();
        assert!(
            err.contains("insufficient available synapse"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn check_low_water_triggers_below_mark() {
        let mut w = AgentComputeWallet::birth_endowment("agent:test");
        w.synapse_balance = 5_000_000; // below LOW_WATER_MARK (10_000_000)
        let d = make_daemon(w);
        let warn = d.check_low_water().await;
        assert!(warn.is_some());
        assert!(matches!(warn.unwrap().severity, WarningSeverity::Warning));
    }

    #[tokio::test]
    async fn check_low_water_critical_at_zero() {
        let mut w = AgentComputeWallet::birth_endowment("agent:test");
        w.synapse_balance = 0;
        let d = make_daemon(w);
        let warn = d.check_low_water().await.unwrap();
        assert!(matches!(warn.severity, WarningSeverity::Critical));
    }

    #[tokio::test]
    async fn check_low_water_none_above_mark() {
        let w = AgentComputeWallet::birth_endowment("agent:test");
        // birth endowment (86M) is well above LOW_WATER_MARK (10M)
        let d = make_daemon(w);
        assert!(d.check_low_water().await.is_none());
    }
}
