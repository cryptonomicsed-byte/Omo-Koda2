use crate::identity::odu::OduIdentity;

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
    pub fn derive(odu: &OduIdentity, tier: u8) -> Self {
        let mask_index = (odu.primary_index as usize) % MASKS.len();
        let mask = MASKS[mask_index].to_string();

        let mood_index = (tier as usize).min(MOODS.len() - 1);
        let mood = MOODS[mood_index].to_string();

        Self { mask, mood }
    }
}
