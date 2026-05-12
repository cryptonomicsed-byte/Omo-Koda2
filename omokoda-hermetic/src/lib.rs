#[derive(Debug, Clone)]
pub struct HermeticState {
    fingerprint_bytes: [u8; 32],
    think_depth: f64,
    cooldown_ms: u64,
}

impl HermeticState {
    pub fn from_seed(name: &str, timestamp: u64) -> Self {
        assert!(!name.is_empty(), "name cannot be empty");

        let mut hasher = blake3::Hasher::new();
        hasher.update(name.as_bytes());
        hasher.update(&timestamp.to_le_bytes());
        let digest = hasher.finalize();

        let mut fingerprint_bytes = [0u8; 32];
        fingerprint_bytes.copy_from_slice(digest.as_bytes());

        let think_depth_raw = u64::from_le_bytes(fingerprint_bytes[0..8].try_into().unwrap());
        let cooldown_raw = u64::from_le_bytes(fingerprint_bytes[8..16].try_into().unwrap());

        Self {
            fingerprint_bytes,
            think_depth: think_depth_raw as f64 / u64::MAX as f64,
            cooldown_ms: cooldown_raw,
        }
    }

    pub fn fingerprint(&self) -> String {
        blake3::Hash::from(self.fingerprint_bytes)
            .to_hex()
            .to_string()
    }

    pub fn think_abstraction_depth(&self) -> f64 {
        self.think_depth
    }

    pub fn act_cooldown_ms(&self) -> u64 {
        self.cooldown_ms
    }
}
