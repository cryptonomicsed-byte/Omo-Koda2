//! SecurityReport — the output of the oso-security analyzer.

use serde::{Deserialize, Serialize};

/// The complete security analysis result.
///
/// `risk_score` range: 0.0 = fully clean, 1.0 = critical failures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityReport {
    /// True only when there are no capability errors, no formal errors,
    /// and (in strict mode) no resource warnings.
    pub passed: bool,

    /// Errors from capability checking (CAP1–CAP3).
    pub capability_errors: Vec<String>,

    /// Warnings from resource bounds checking (RES1–RES3).
    pub resource_warnings: Vec<String>,

    /// Errors from formal analysis (FORM1–FORM3).
    pub formal_errors: Vec<String>,

    /// Composite risk score: 0.0 (clean) → 1.0 (critical).
    pub risk_score: f32,
}
