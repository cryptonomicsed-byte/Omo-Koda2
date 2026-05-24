/// BIPỌ̀N39 vendored CI stub.
///
/// Used in CI when omo-koda/Bipon39-Rust is not accessible.
/// API is kept compatible with what omokoda-core and omokoda-hermetic expect.

// ── Macro (7 Orisha groupings) ────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Macro {
    Esu,
    Sango,
    Osun,
    Yemoja,
    Oya,
    Ogun,
    Obatala,
}

impl Macro {
    pub fn name(&self) -> &'static str {
        match self {
            Macro::Esu => "Esu",
            Macro::Sango => "Sango",
            Macro::Osun => "Osun",
            Macro::Yemoja => "Yemoja",
            Macro::Oya => "Oya",
            Macro::Ogun => "Ogun",
            Macro::Obatala => "Obatala",
        }
    }

    pub fn from_name(s: &str) -> Option<Self> {
        match s {
            "Esu" => Some(Macro::Esu),
            "Sango" => Some(Macro::Sango),
            "Osun" => Some(Macro::Osun),
            "Yemoja" => Some(Macro::Yemoja),
            "Oya" => Some(Macro::Oya),
            "Ogun" => Some(Macro::Ogun),
            "Obatala" => Some(Macro::Obatala),
            _ => None,
        }
    }

    fn all() -> [Macro; 7] {
        [
            Macro::Esu,
            Macro::Sango,
            Macro::Osun,
            Macro::Yemoja,
            Macro::Oya,
            Macro::Ogun,
            Macro::Obatala,
        ]
    }
}

// ── MacroDistribution ─────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct MacroDistribution {
    pub counts: [(Macro, usize); 7],
    pub total: usize,
}

// ── ElementalVector ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ElementalVector {
    pub fire: usize,
    pub water: usize,
    pub earth: usize,
    pub air: usize,
    pub ether: usize,
}

// ── PersonalityProfile ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct PersonalityProfile {
    pub macro_distribution: MacroDistribution,
    pub macro_percentages: [(Macro, f64); 7],
    pub elemental_signature: ElementalVector,
    pub dominant_orisha: Macro,
    pub ritual_suggestions: Vec<String>,
    pub personality_summary: String,
}

// ── WordlistEntry stub ────────────────────────────────────────────────────

pub struct WordlistEntry {
    pub array_index: usize,
}

// ── Core functions ────────────────────────────────────────────────────────

/// Encode entropy bytes into a 33-word mnemonic phrase.
pub fn entropy_to_mnemonic(entropy: &[u8]) -> Result<Vec<String>, String> {
    if entropy.is_empty() {
        return Err("empty entropy".to_string());
    }
    let words: Vec<String> = (0..33)
        .map(|i| {
            let byte = entropy[i % entropy.len()];
            format!("b{:03}", (byte as usize + i * 7) % 256)
        })
        .collect();
    Ok(words)
}

/// Split a mnemonic string into its constituent words.
pub fn split_mnemonic(phrase: &str) -> Vec<&str> {
    phrase.split_whitespace().collect()
}

/// Derive a 64-byte seed from mnemonic words and passphrase.
pub fn mnemonic_to_seed(words: &[&str], passphrase: &str) -> Result<Vec<u8>, String> {
    if words.is_empty() {
        return Err("empty mnemonic".to_string());
    }
    let mut seed = vec![0u8; 64];
    let input = format!("{}{}", words.join(" "), passphrase);
    for (i, byte) in input.bytes().enumerate() {
        seed[i % 64] ^= byte.wrapping_add(i as u8);
    }
    Ok(seed)
}

/// Look up a mnemonic word by its encoded form.
pub fn entry_by_encoding(word: &str) -> Result<WordlistEntry, String> {
    let index = if let Some(n) = word.strip_prefix('b') {
        n.parse::<usize>().unwrap_or(0) % 256
    } else {
        word.bytes().fold(0usize, |acc, b| (acc + b as usize) % 256)
    };
    Ok(WordlistEntry { array_index: index })
}

/// Build a personality profile from a whitespace-separated mnemonic.
pub fn personality_profile(mnemonic: &str) -> Result<PersonalityProfile, String> {
    let macros = Macro::all();
    let hash: usize = mnemonic.bytes().map(|b| b as usize).sum();
    let counts: [(Macro, usize); 7] = macros.map(|m| (m, (hash + m.name().len()) % 10 + 1));
    let total: usize = counts.iter().map(|(_, c)| c).sum();
    let percentages: [(Macro, f64); 7] =
        counts.map(|(m, c)| (m, (c as f64 / total as f64) * 100.0));
    let dominant = macros[hash % 7];

    Ok(PersonalityProfile {
        macro_distribution: MacroDistribution { counts, total },
        macro_percentages: percentages,
        elemental_signature: ElementalVector {
            fire: hash % 5 + 1,
            water: (hash / 5) % 5 + 1,
            earth: (hash / 25) % 5 + 1,
            air: (hash / 125) % 5 + 1,
            ether: (hash / 625) % 5 + 1,
        },
        dominant_orisha: dominant,
        ritual_suggestions: vec![format!("Align with {}", dominant.name())],
        personality_summary: format!("{} leads with elemental tone.", dominant.name()),
    })
}
