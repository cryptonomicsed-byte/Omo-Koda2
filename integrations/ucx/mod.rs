//! Omo-Koda2 / UCX integration — HTTP client bindings for the Universal Compute Exchange.
//!
//! Module layout:
//!   compute_tools   — submit jobs, offer local compute
//!   provider_tools  — register/deregister/list providers via Vantage
//!   receipt         — fetch + store ComputeReceipts, emit ARP receipts
//!   policy          — agent compute budget + workload policy

pub mod compute_tools;
pub mod policy;
pub mod provider_tools;
pub mod receipt;

pub use compute_tools::{ucx_request_compute, ucx_offer_compute};
pub use policy::ComputePolicy;
pub use receipt::{ComputeJobRecord, fetch_and_emit};
