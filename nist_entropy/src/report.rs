use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TestResult {
    Pass,
    Fail { reason: String },
}

impl TestResult {
    pub fn is_pass(&self) -> bool {
        matches!(self, TestResult::Pass)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NistReport {
    pub frequency: TestResult,
    pub runs: TestResult,
    pub longest_run: TestResult,
    pub avalanche: TestResult,
    pub all_passed: bool,
}

impl NistReport {
    pub fn new(
        frequency: TestResult,
        runs: TestResult,
        longest_run: TestResult,
        avalanche: TestResult,
    ) -> Self {
        let all_passed = [&frequency, &runs, &longest_run, &avalanche]
            .iter()
            .all(|r| r.is_pass());
        Self {
            frequency,
            runs,
            longest_run,
            avalanche,
            all_passed,
        }
    }
}
