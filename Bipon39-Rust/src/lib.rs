use serde::{Deserialize, Serialize};

/// One of 7 primary Orishas assigned at birth from the Odu seed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Orisha {
    index: u8,
}

impl Orisha {
    pub fn name(&self) -> &'static str {
        match self.index % 7 {
            0 => "Esu",
            1 => "Obatala",
            2 => "Osun",
            3 => "Yemoja",
            4 => "Ogun",
            5 => "Sango",
            _ => "Oya",
        }
    }
}

/// Personality profile derived from a BIPỌ̀N39 mnemonic.
/// Fields map to behavioral tendencies seeded at birth.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PersonalityProfile {
    pub curiosity: f32,
    pub creativity: f32,
    pub discipline: f32,
    pub empathy: f32,
    pub resilience: f32,
    pub intuition: f32,
    pub sovereignty: f32,
    pub dominant_orisha: Orisha,
    pub personality_summary: String,
}

impl Default for PersonalityProfile {
    fn default() -> Self {
        Self {
            curiosity: 0.5,
            creativity: 0.5,
            discipline: 0.5,
            empathy: 0.5,
            resilience: 0.5,
            intuition: 0.5,
            sovereignty: 0.5,
            dominant_orisha: Orisha { index: 0 },
            personality_summary: "Balanced".to_string(),
        }
    }
}

pub struct Entry {
    pub array_index: u32,
    pub word: String,
}

/// Derives a personality profile from the full mnemonic string.
pub fn personality_profile(mnemonic: &str) -> Result<PersonalityProfile, String> {
    if mnemonic.is_empty() {
        return Ok(PersonalityProfile::default());
    }
    let hash = blake3::hash(mnemonic.as_bytes());
    let bytes = hash.as_bytes();
    let orisha_idx = bytes[7] % 7;
    let summaries = [
        "Curious and open",
        "Highly creative",
        "Disciplined and focused",
        "Deeply empathic",
        "Highly resilient",
        "Strongly intuitive",
        "Sovereign and independent",
    ];
    let summary = summaries[(bytes[8] % 7) as usize].to_string();
    Ok(PersonalityProfile {
        curiosity: bytes[0] as f32 / 255.0,
        creativity: bytes[1] as f32 / 255.0,
        discipline: bytes[2] as f32 / 255.0,
        empathy: bytes[3] as f32 / 255.0,
        resilience: bytes[4] as f32 / 255.0,
        intuition: bytes[5] as f32 / 255.0,
        sovereignty: bytes[6] as f32 / 255.0,
        dominant_orisha: Orisha { index: orisha_idx },
        personality_summary: summary,
    })
}

/// Converts raw entropy bytes to a list of mnemonic words.
/// BIPỌ̀N39 uses 8 bits per word (256-word vocabulary):
/// 32 bytes entropy → 32 content words + 1 checksum word = 33 total.
pub fn entropy_to_mnemonic(entropy: &[u8]) -> Result<Vec<String>, String> {
    let mut words: Vec<String> = entropy
        .iter()
        .enumerate()
        .map(|(i, &b)| format!("odu-{:02x}-{:02x}", b, i as u8))
        .collect();
    // Append checksum word derived from all content bytes
    let checksum: u8 = entropy.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
    words.push(format!("odu-chk-{:02x}", checksum));
    Ok(words)
}

/// Derives a 64-byte seed from mnemonic word slices and an optional passphrase.
pub fn mnemonic_to_seed(words: &[&str], passphrase: &str) -> Result<[u8; 64], String> {
    let input = format!("{}{}", words.join(" "), passphrase);
    let hash = blake3::hash(input.as_bytes());
    let mut seed = [0u8; 64];
    seed[..32].copy_from_slice(hash.as_bytes());
    let hash2 = blake3::hash(hash.as_bytes());
    seed[32..].copy_from_slice(hash2.as_bytes());
    Ok(seed)
}

/// Splits a mnemonic string into individual word slices borrowed from the input.
pub fn split_mnemonic(mnemonic: &str) -> Vec<&str> {
    mnemonic.split_whitespace().collect()
}

/// Looks up a wordlist entry by its encoded form.
pub fn entry_by_encoding(word: &str) -> Result<Entry, String> {
    let idx = if let Some(hex) = word.strip_prefix("odu-") {
        u32::from_str_radix(hex, 16).unwrap_or(0)
    } else {
        let hash = blake3::hash(word.as_bytes());
        let b = hash.as_bytes();
        u32::from_le_bytes([b[0], b[1], b[2], b[3]])
    };
    Ok(Entry {
        array_index: idx % 256,
        word: word.to_string(),
    })
}
