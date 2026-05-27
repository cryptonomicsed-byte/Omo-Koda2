//! Poison Radar — Garden address safety scanning.
//! From vanity-cloakseed. Detects address poisoning attacks.

/// A known-safe address entry in the agent's address book.
#[derive(Debug, Clone)]
pub struct SafeAddress {
    pub label: String,
    pub address: String,
    pub verified_at: u64,
}

/// Poison Radar — scans for address poisoning in the agent's Garden interactions.
pub struct PoisonRadar {
    safe_addresses: Vec<SafeAddress>,
}

impl PoisonRadar {
    pub fn new() -> Self {
        Self { safe_addresses: Vec::new() }
    }

    /// Register a verified safe address.
    pub fn register(&mut self, label: impl Into<String>, address: impl Into<String>, timestamp: u64) {
        self.safe_addresses.push(SafeAddress {
            label: label.into(),
            address: address.into(),
            verified_at: timestamp,
        });
    }

    /// Check if an address looks like a poison clone of a known safe address.
    /// Returns Some(similar_address) if suspicious, None if clean.
    pub fn scan(&self, candidate: &str) -> Option<&SafeAddress> {
        self.safe_addresses.iter().find(|safe| {
            candidate != safe.address && self.looks_similar(candidate, &safe.address)
        })
    }

    /// Heuristic similarity: same prefix or same suffix, different middle.
    fn looks_similar(&self, a: &str, b: &str) -> bool {
        if a.len() < 8 || b.len() < 8 || a.len() != b.len() {
            return false;
        }
        let prefix_match = a[..4] == b[..4];
        let suffix_match = a[a.len()-4..] == b[b.len()-4..];
        (prefix_match || suffix_match) && a != b
    }

    pub fn known_safe(&self, address: &str) -> bool {
        self.safe_addresses.iter().any(|s| s.address == address)
    }
}

impl Default for PoisonRadar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_address_is_safe() {
        let mut radar = PoisonRadar::new();
        radar.register("alice", "0xabcd1234efgh5678", 0);
        assert!(radar.known_safe("0xabcd1234efgh5678"));
        assert!(!radar.known_safe("0xabcd1234efgh9999"));
    }

    #[test]
    fn poison_address_detected() {
        let mut radar = PoisonRadar::new();
        radar.register("alice", "0xabcd000011119999", 0);
        // Same prefix/suffix, different middle — suspicious
        let result = radar.scan("0xabcd999900009999");
        assert!(result.is_some());
    }

    #[test]
    fn exact_match_not_flagged() {
        let mut radar = PoisonRadar::new();
        radar.register("alice", "0xabcd000011119999", 0);
        assert!(radar.scan("0xabcd000011119999").is_none());
    }
}
