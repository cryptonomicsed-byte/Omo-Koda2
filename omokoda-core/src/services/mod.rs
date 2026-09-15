//! Background daemon services for Omo-Koda2.

pub mod walletd;

pub use walletd::{ComputeWarning, Walletd, WalletdConfig, WarningSeverity};
