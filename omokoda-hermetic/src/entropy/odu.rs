use ifascript::odu::get_odu;

pub struct OduEntropy;

impl OduEntropy {
    /// Maps a 256-index Odu (0-255) to a deterministic entropy byte.
    pub fn get_entropy(odu_index: u8) -> u8 {
        let odu = get_odu(odu_index);
        // Deterministic mapping based on the Odu's cosmological properties
        let mut hasher = blake3::Hasher::new();
        hasher.update(odu.name.as_bytes());
        hasher.update(odu.archetype.as_bytes());
        hasher.finalize().as_bytes()[0]
    }

    /// Generates a 32-byte seed from a mnemonic's Odu indices.
    /// This replaces the raw BLAKE3 as the hermetic seed source.
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
