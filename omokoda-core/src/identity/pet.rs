use crate::identity::odu::OduIdentity;
use omokoda_hermetic::HermeticState;

pub const MASKS: &[&str] = &[
    "(=^.^=)",
    "(๑•ᴗ•๑)",
    "(◕‿◕✿)",
    "(｡♥‿♥｡)",
    "(✿◠‿◠)",
    "(╯◕_◕)╯",
    "ʕ•ᴥ•ʔ",
    "(^._.^)",
    "(❍ᴥ❍ʋ)",
    "(V•ᴥ•V)",
    "(ΦωΦ)",
    "(ΘεΘ;)",
    "( ^..^ )",
    "( ⓛ ω ⓛ )",
    "(ﾐ ᵕ ω ᵕ ﾐ)",
    "(ㅇㅅㅇ❀)",
    "( =ω= )",
    "( ´艸｀)",
    "( = ^ • ω • ^ = )",
    "(๑✪ω✪๑)",
    "( ⓛ ܫ ⓛ )",
    "( = ① ω ① = )",
    "( = ᆺ = )",
    "( = ⊝ = )",
    "( = ᴥ = )",
    "(= ^ ◡ ^ =)",
    "(= ᵒ ᆺ ᵒ =)",
    "( ﾐ ⫑ ﻌ ⫒ ﾐ )",
    "(=^-ω-^=)",
    "(^=◕ᴥ◕=^)",
    "(＾• ω •＾)",
];

pub const MOODS: &[&str] = &[
    "newborn", "curious", "focused", "playful", "serene", "wise", "sovereign",
];

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PetIdentity {
    pub mask: String,
    pub mood: String,
}

impl PetIdentity {
    pub fn derive(odu: &OduIdentity, hermetic: &HermeticState, tier: u8) -> Self {
        // Mask evolves with tier
        let mask_index = (odu.primary_index as usize + tier as usize) % MASKS.len();
        let mask = MASKS[mask_index].to_string();

        // Mood is influenced by Hermetic principles: Mentalism, Polarity, Vibration
        let h_score = (hermetic.mentalism() + hermetic.polarity() + hermetic.vibration()) / 3.0;
        
        // Combine Hermetic score with tier to select mood
        // This ensures mood reflects both the "soul" (Hermetic) and "growth" (Tier)
        let mood_index = ((h_score * (MOODS.len() - 1) as f64) as usize + tier as usize) % MOODS.len();
        let mood = MOODS[mood_index].to_string();

        Self { mask, mood }
    }

    pub fn pet(&self) -> &str {
        &self.mask
    }
}

