/// NIST SP 800-22 entropy validation (pure Rust subset).
/// Implements frequency, runs, longest-run, and avalanche tests without
/// requiring the dieharder C library. A full dieharder FFI binding can be
/// added later by wiring up `build.rs` + bindgen.

pub mod report;
pub mod validator;

pub use report::{NistReport, TestResult};
pub use validator::validate_entropy_seed;
