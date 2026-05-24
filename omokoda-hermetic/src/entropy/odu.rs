pub struct OduEntropy;

impl OduEntropy {
    /// Maps a 256-index Odu (0-255) to a deterministic entropy byte.
    pub fn get_entropy(odu_index: u8) -> u8 {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"omokoda:odu:");
        hasher.update(&[odu_index]);
        hasher.finalize().as_bytes()[0]
    }

    /// Generates a 32-byte seed from a mnemonic's Odu indices.
    pub fn generate_hermetic_seed(odu_indices: &[u8]) -> [u8; 32] {
        let mut seed = [0u8; 32];
        for (i, &idx) in odu_indices.iter().enumerate() {
            if i >= 32 {
                break;
            }
            seed[i] = Self::get_entropy(idx);
        }
        seed
    }
}
