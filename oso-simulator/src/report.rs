//! SimReport — the final output of a simulation run.

use serde::Serialize;

/// Complete simulation report.
///
/// Fields mirror the task spec + the Rust OsoRunResult in oso-sdk-ts/src/types.ts.
#[derive(Debug, Serialize)]
pub struct SimReport {
    /// Final MockOsovmState snapshot (balances, jobs, kv keys, etc.)
    pub final_state: serde_json::Value,

    /// Deterministic receipt hashes emitted during execution.
    pub receipts_emitted: Vec<String>,

    /// Total ASE deducted from the caller agent.
    pub ase_spent: u64,

    /// Total ASE credited to the caller agent.
    pub ase_earned: u64,

    /// Error message if execution failed, None on success.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
