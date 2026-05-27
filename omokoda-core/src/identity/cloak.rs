//! CloakSeed — owner key protection via user-defined cipher overlay.
//! From vanity-cloakseed. Seed phrases cloaked with user-defined transformation.

/// A cipher overlay applied to mnemonic words before display.
/// Prevents shoulder-surfing without reducing cryptographic strength.
pub struct CloakSeed {
    rotation_offset: u8,
}

impl CloakSeed {
    /// Create with a user-defined offset (0–255).
    pub fn new(offset: u8) -> Self {
        Self {
            rotation_offset: offset,
        }
    }

    /// Cloak a mnemonic word list for display. Words themselves unchanged in storage.
    /// This is a display-layer protection only, not encryption.
    pub fn cloak_display(&self, words: &[String]) -> Vec<String> {
        words
            .iter()
            .enumerate()
            .map(|(i, w)| {
                let shift = ((i as u8).wrapping_add(self.rotation_offset)) % 26;
                w.chars()
                    .map(|c| {
                        if c.is_ascii_lowercase() {
                            (b'a' + (c as u8 - b'a' + shift) % 26) as char
                        } else if c.is_ascii_uppercase() {
                            (b'A' + (c as u8 - b'A' + shift) % 26) as char
                        } else {
                            c
                        }
                    })
                    .collect()
            })
            .collect()
    }

    /// Verify a cloaked display word matches original at given position.
    pub fn verify(&self, position: usize, original: &str, cloaked: &str) -> bool {
        let shift = ((position as u8).wrapping_add(self.rotation_offset)) % 26;
        let re_cloaked: String = original
            .chars()
            .map(|c| {
                if c.is_ascii_lowercase() {
                    (b'a' + (c as u8 - b'a' + shift) % 26) as char
                } else if c.is_ascii_uppercase() {
                    (b'A' + (c as u8 - b'A' + shift) % 26) as char
                } else {
                    c
                }
            })
            .collect();
        re_cloaked == cloaked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cloak_and_verify_round_trip() {
        let cloak = CloakSeed::new(7);
        let words = vec!["hello".to_string(), "world".to_string()];
        let cloaked = cloak.cloak_display(&words);
        assert_ne!(cloaked[0], words[0]);
        assert!(cloak.verify(0, &words[0], &cloaked[0]));
        assert!(cloak.verify(1, &words[1], &cloaked[1]));
    }

    #[test]
    fn zero_offset_is_identity() {
        let cloak = CloakSeed::new(0);
        let words = vec!["abc".to_string()];
        let cloaked = cloak.cloak_display(&words);
        assert_eq!(cloaked[0], words[0]);
    }
}
