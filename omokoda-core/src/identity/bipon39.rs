use sha2::{Digest, Sha256};

pub const ROOTS: [&str; 16] = [
    "esu", "sango", "ogun", "oya", "yemoja", "osun", "obatala", "orunmila", "egungun", "ori",
    "ile", "omi", "ina", "afeefe", "igi", "irawo",
];

pub const AFFIXES: [&str; 16] = [
    "gate", "volt", "forge", "stream", "tide", "veil", "crown", "mirror", "path", "seal", "code",
    "sigil", "drum", "thunder", "river", "dawn",
];

pub struct Bipon39;

impl Bipon39 {
    pub fn wordlist() -> Vec<String> {
        let mut list = Vec::with_capacity(256);
        for root in ROOTS.iter() {
            for affix in AFFIXES.iter() {
                list.push(format!("{}-{}", root, affix));
            }
        }
        list
    }

    pub fn wordlist_merkle_root() -> String {
        let words = Self::wordlist();
        let mut hashes: Vec<blake3::Hash> =
            words.iter().map(|w| blake3::hash(w.as_bytes())).collect();

        while hashes.len() > 1 {
            let mut next_level = Vec::new();
            for i in (0..hashes.len()).step_by(2) {
                let mut hasher = blake3::Hasher::new();
                hasher.update(hashes[i].as_bytes());
                if i + 1 < hashes.len() {
                    hasher.update(hashes[i + 1].as_bytes());
                } else {
                    hasher.update(hashes[i].as_bytes());
                }
                next_level.push(hasher.finalize());
            }
            hashes = next_level;
        }

        hashes[0].to_hex().to_string()
    }

    pub fn verify_wordlist_integrity() -> bool {
        // Expected Merkle root for the canonical 256-token wordlist
        let expected = "f266047fcc5713d0ceb6b1c7fd225a8ae71416613a8dc7a714f9d1481763fd37";
        Self::wordlist_merkle_root() == expected
    }

    pub fn entropy_to_mnemonic(entropy: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(entropy);
        let checksum = hasher.finalize();

        // 256 bits entropy + 8 bits checksum (first byte) = 264 bits = 33 words (8 bits each)
        // Actually BIP-39 uses ENT / 32 bits for checksum.
        // For 256 bits, it's 8 bits (1 byte).
        let checksum_byte = checksum[0];

        let mut combined = Vec::with_capacity(entropy.len() + 1);
        combined.extend_from_slice(entropy);
        combined.push(checksum_byte);

        let words = Self::wordlist();
        let mnemonic_words: Vec<String> = combined
            .iter()
            .map(|&b| words[b as usize].clone())
            .collect();

        mnemonic_words.join(" ")
    }

    pub fn mnemonic_to_indices(mnemonic: &str) -> Result<Vec<u8>, String> {
        let words = Self::wordlist();
        let mut indices = Vec::new();
        for word in mnemonic.split_whitespace() {
            let index = words
                .iter()
                .position(|w| w == word)
                .ok_or_else(|| format!("Invalid word in mnemonic: {}", word))?;
            indices.push(index as u8);
        }
        Ok(indices)
    }

    pub fn mnemonic_to_seed(
        mnemonic: &str,
        agent_id: &str,
        birth_timestamp: u64,
        chain_id: &str,
    ) -> [u8; 32] {
        use argon2::{Algorithm, Argon2, Params, PasswordHasher, Version};

        let mut hasher = blake3::Hasher::new();
        hasher.update(agent_id.as_bytes());
        hasher.update(&birth_timestamp.to_le_bytes());
        hasher.update(chain_id.as_bytes());
        let salt = hasher.finalize();

        let params = Params::new(65536, 3, 1, Some(32)).expect("invalid argon2 params");
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let mut output = [0u8; 32];
        argon2
            .hash_password_into(mnemonic.as_bytes(), salt.as_bytes(), &mut output)
            .expect("Argon2id failed");
        output
    }

    pub fn get_odu_index(indices: &[u8]) -> u8 {
        indices.iter().fold(0, |acc, &x| acc ^ x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wordlist_length() {
        assert_eq!(Bipon39::wordlist().len(), 256);
    }

    #[test]
    fn test_wordlist_integrity() {
        let root = Bipon39::wordlist_merkle_root();
        println!("WORDLIST MERKLE ROOT: {}", root);
        assert!(Bipon39::verify_wordlist_integrity());
    }

    #[test]
    fn test_mnemonic_roundtrip() {
        let entropy = [0u8; 32];
        let mnemonic = Bipon39::entropy_to_mnemonic(&entropy);
        let indices = Bipon39::mnemonic_to_indices(&mnemonic).unwrap();
        assert_eq!(indices.len(), 33);
        assert_eq!(&indices[..32], &entropy);
    }
}
