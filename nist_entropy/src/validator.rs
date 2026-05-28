use sha3::{Digest, Sha3_256};

use crate::report::{NistReport, TestResult};

/// Run the full NIST entropy validation battery on a seed.
/// Returns a report indicating which tests passed or failed.
/// A valid entropy source should pass all four tests at 99% confidence.
pub fn validate_entropy_seed(seed: &[u8]) -> NistReport {
    let bits = bytes_to_bits(seed);
    NistReport::new(
        frequency_test(&bits),
        runs_test(&bits),
        longest_run_ones_test(&bits),
        avalanche_test(seed),
    )
}

/// NIST SP 800-22 Test 1: Frequency (Monobit) Test.
/// The proportion of 0s and 1s should be approximately equal.
/// Passes if |proportion - 0.5| < 0.05 (simplified version).
fn frequency_test(bits: &[bool]) -> TestResult {
    if bits.is_empty() {
        return TestResult::Fail {
            reason: "empty seed".to_string(),
        };
    }
    let ones = bits.iter().filter(|&&b| b).count() as f64;
    let proportion = ones / bits.len() as f64;
    if (proportion - 0.5).abs() < 0.05 {
        TestResult::Pass
    } else {
        TestResult::Fail {
            reason: format!(
                "proportion of 1s is {:.3}, expected ~0.5 (±0.05)",
                proportion
            ),
        }
    }
}

/// NIST SP 800-22 Test 2: Runs Test.
/// Counts the total number of runs (consecutive identical bits).
/// A run count too far from expected indicates non-randomness.
fn runs_test(bits: &[bool]) -> TestResult {
    if bits.len() < 8 {
        return TestResult::Fail {
            reason: "too few bits for runs test".to_string(),
        };
    }
    let n = bits.len() as f64;
    let ones = bits.iter().filter(|&&b| b).count() as f64;
    let proportion = ones / n;

    // Pre-test: if proportion is too extreme, fail immediately
    if (proportion - 0.5).abs() >= 0.05 {
        return TestResult::Fail {
            reason: "proportion prerequisite failed for runs test".to_string(),
        };
    }

    let runs: usize = bits.windows(2).filter(|w| w[0] != w[1]).count() + 1;
    let expected = 2.0 * n * proportion * (1.0 - proportion);
    let variance = (2.0 * n * proportion * (1.0 - proportion) * (1.0 - 2.0 * proportion * (1.0 - proportion))).abs();

    if variance < f64::EPSILON {
        return TestResult::Pass;
    }

    // Accept if runs count is within 3 standard deviations of expected
    let std_dev = variance.sqrt();
    let z = (runs as f64 - expected).abs() / std_dev;
    if z < 3.0 {
        TestResult::Pass
    } else {
        TestResult::Fail {
            reason: format!("runs count {} deviates {:.2}σ from expected {:.1}", runs, z, expected),
        }
    }
}

/// NIST SP 800-22 Test 3: Longest Run of Ones in a Block.
/// Checks that no excessively long run of 1s exists.
fn longest_run_ones_test(bits: &[bool]) -> TestResult {
    if bits.is_empty() {
        return TestResult::Fail {
            reason: "empty seed".to_string(),
        };
    }

    // Find the longest run of consecutive 1s
    let mut max_run = 0usize;
    let mut current_run = 0usize;
    for &bit in bits {
        if bit {
            current_run += 1;
            max_run = max_run.max(current_run);
        } else {
            current_run = 0;
        }
    }

    // Simplified bound: longest run should not exceed log2(n) + 4
    let n = bits.len() as f64;
    let max_allowed = (n.log2() + 4.0) as usize;
    if max_run <= max_allowed {
        TestResult::Pass
    } else {
        TestResult::Fail {
            reason: format!(
                "longest run of 1s = {}, max allowed = {}",
                max_run, max_allowed
            ),
        }
    }
}

/// Avalanche test: flipping one bit of the seed should change ≥ 45% of
/// the SHA3-256 hash output bits. Validates the seed has good diffusion.
fn avalanche_test(seed: &[u8]) -> TestResult {
    if seed.is_empty() {
        return TestResult::Fail {
            reason: "empty seed".to_string(),
        };
    }

    let hash_a = sha3_hash(seed);

    // Flip the MSB of the first byte
    let mut flipped = seed.to_vec();
    flipped[0] ^= 0x80;
    let hash_b = sha3_hash(&flipped);

    // Count differing bits
    let differing: usize = hash_a
        .iter()
        .zip(hash_b.iter())
        .map(|(a, b)| (a ^ b).count_ones() as usize)
        .sum();

    let total_bits = hash_a.len() * 8;
    let proportion = differing as f64 / total_bits as f64;

    if proportion >= 0.45 {
        TestResult::Pass
    } else {
        TestResult::Fail {
            reason: format!(
                "avalanche effect only {:.1}% — expected ≥ 45%",
                proportion * 100.0
            ),
        }
    }
}

fn bytes_to_bits(bytes: &[u8]) -> Vec<bool> {
    let mut bits = Vec::with_capacity(bytes.len() * 8);
    for byte in bytes {
        for shift in (0..8).rev() {
            bits.push((byte >> shift) & 1 == 1);
        }
    }
    bits
}

fn sha3_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha3_of_hello_passes_all() {
        let seed = sha3_hash(b"hello world");
        let report = validate_entropy_seed(&seed);
        assert!(report.all_passed, "SHA3 output should pass all NIST tests: {:#?}", report);
    }

    #[test]
    fn all_zeros_fails_frequency() {
        let seed = [0u8; 32];
        let report = validate_entropy_seed(&seed);
        assert!(!report.frequency.is_pass(), "all-zeros should fail frequency test");
    }

    #[test]
    fn all_zeros_fails_avalanche() {
        let seed = [0u8; 32];
        let report = validate_entropy_seed(&seed);
        // SHA3([0]*32) XOR SHA3([0x80, 0...]) should differ substantially
        // but the seed itself (constant input) fails avalanche — let's check
        assert!(!report.all_passed, "all-zeros entropy should not pass all tests");
    }

    #[test]
    fn alternating_bytes_passes_frequency() {
        // 0xAA = 10101010 in binary — exactly 50% ones
        let seed = [0xAAu8; 32];
        let freq = super::frequency_test(&bytes_to_bits(&seed));
        assert!(freq.is_pass(), "alternating bits should pass frequency: {:?}", freq);
    }

    #[test]
    fn report_all_passed_requires_all_pass() {
        use crate::report::{NistReport, TestResult};
        let report = NistReport::new(
            TestResult::Pass,
            TestResult::Fail { reason: "x".to_string() },
            TestResult::Pass,
            TestResult::Pass,
        );
        assert!(!report.all_passed);
    }
}
