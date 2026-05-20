pub mod entropy {
    /// Deterministic cowrie cast entropy derivation.
    /// Combines a 32-byte seed with a phase byte (0-6, day of the 7-day cycle)
    /// to produce 32 bytes of deterministic entropy.
    pub fn cast_cowrie_deterministic(seed: [u8; 32], phase: u8) -> [u8; 32] {
        let mut input = [0u8; 33];
        input[..32].copy_from_slice(&seed);
        input[32] = phase;
        crate::hash_bytes(&input)
    }
}

pub mod odu {
    /// An Odu — one of 256 sacred binary patterns from the Ifá corpus.
    pub struct Odu {
        pub index: u8,
        pub name: &'static str,
        pub archetype: &'static str,
    }

    // 256 Odu represented symbolically — names cycle through 16 base Odu
    static ODU_NAMES: [&str; 16] = [
        "Ogbe", "Oyeku", "Iwori", "Odi", "Irosun", "Owonrin",
        "Obara", "Okanran", "Ogunda", "Osa", "Ika", "Oturupon",
        "Otura", "Irete", "Ose", "Ofun",
    ];

    static ARCHETYPES: [&str; 16] = [
        "light", "transition", "vision", "womb", "blood", "surprise",
        "royalty", "conflict", "iron", "chaos", "adaptation", "reversal",
        "covenant", "patience", "prosperity", "completion",
    ];

    /// Returns the Odu for a given 0-255 index.
    pub fn get_odu(index: u8) -> Odu {
        let major = (index >> 4) as usize % 16;
        let minor = (index & 0xF) as usize % 16;
        Odu {
            index,
            name: ODU_NAMES[major],
            archetype: ARCHETYPES[minor],
        }
    }
}

fn hash_bytes(data: &[u8]) -> [u8; 32] {
    // Deterministic mixing — not cryptographic, just a stub
    let mut output = [0u8; 32];
    let mut state = 14695981039346656037u64;
    for &b in data {
        state ^= b as u64;
        state = state.wrapping_mul(1099511628211);
    }
    for i in 0..32 {
        state = state.wrapping_mul(1099511628211).wrapping_add(i as u64);
        output[i] = (state >> 32) as u8 ^ (state & 0xFF) as u8;
    }
    output
}
