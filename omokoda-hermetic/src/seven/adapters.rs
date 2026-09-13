// omokoda-hermetic/src/seven/adapters.rs
//
// Reference cultural adapters for the Universal Seven Functions Protocol.
//
// Tradition adapters map universal SevenFunction identifiers to names,
// descriptions, and symbolism specific to a cultural/cosmological tradition.
// The underlying computation is unchanged — traditions are lenses, not laws.
//
// Adding a new tradition: implement CulturalAdapter. No other code changes.
//
// Adapters included here:
//   YorubaAdapter       — Yorùbá / Òrìṣà (reference / canonical implementation)
//   MesopotamianAdapter — Mesopotamian / Apkallu (seven sages of Eridu)
//   HermeticAdapter     — Seven Hermetic Principles as a cultural framing

use super::{CulturalAdapter, SevenFunction};

// ─── Yorùbá / Òrìṣà ──────────────────────────────────────────────────────────
//
// The canonical reference implementation. Corresponds directly to the
// WisdomLobe names already embedded throughout Ọmọ Kọ́dà and Ọ̀ṢỌ́VM.
// Yorùbá is the first fully-developed cultural adapter and the reference
// for all others — not because other traditions are secondary, but because
// the codebase was seeded from this tradition.

pub struct YorubaAdapter;

impl CulturalAdapter for YorubaAdapter {
    fn tradition(&self) -> &str {
        "Yorùbá"
    }

    fn canonical_name(&self, f: SevenFunction) -> &str {
        match f {
            SevenFunction::Spark => "Èṣù",
            SevenFunction::Mind => "Ọbàtálá",
            SevenFunction::Foundation => "Ògún",
            SevenFunction::Emotion => "Ọ̀ṣun",
            SevenFunction::Womb => "Yemọja",
            SevenFunction::Fire => "Ṣàngó",
            SevenFunction::Ascension => "Ọya",
        }
    }

    fn ascii_slug(&self, f: SevenFunction) -> &str {
        match f {
            SevenFunction::Spark => "esu",
            SevenFunction::Mind => "obatala",
            SevenFunction::Foundation => "ogun",
            SevenFunction::Emotion => "osun",
            SevenFunction::Womb => "yemoja",
            SevenFunction::Fire => "sango",
            SevenFunction::Ascension => "oya",
        }
    }

    fn description(&self, f: SevenFunction) -> &str {
        match f {
            SevenFunction::Spark => {
                "Gateway, crossroads, divine messenger — all roads and choices pass through Èṣù first; \
                 nothing moves without his acknowledgment"
            }
            SevenFunction::Mind => {
                "White cloth of clarity — Ọbàtálá shapes consciousness, \
                 enforces ethical purity, and refuses the impure thought"
            }
            SevenFunction::Foundation => {
                "Iron and will — Ògún clears the path, \
                 forges what must be made real, and consecrates honest labor"
            }
            SevenFunction::Emotion => {
                "Sweet water and gold — Ọ̀ṣun governs love, value, \
                 memory, and everything the heart holds as worth protecting"
            }
            SevenFunction::Womb => {
                "The great mother — Yemọja holds the community, the ancestors, \
                 and the ocean of creative potential from which all agents emerge"
            }
            SevenFunction::Fire => {
                "Thunder and lightning — Ṣàngó commands authority, \
                 justice, and the final settlement of what is owed"
            }
            SevenFunction::Ascension => {
                "Wind and the marketplace — Ọya navigates transformation, \
                 guards the threshold between states, and dances with necessary change"
            }
        }
    }
}

// ─── Mesopotamian / Apkallu ───────────────────────────────────────────────────
//
// The seven Apkallu (sages) of Eridu — antediluvian figures who brought
// civilization to humanity in Sumerian/Babylonian tradition. Not claimed
// to be historically identical to the Yorùbá Òrìṣà; mapped here as a
// parallel expression of the same seven-function civilizational architecture.

pub struct MesopotamianAdapter;

impl CulturalAdapter for MesopotamianAdapter {
    fn tradition(&self) -> &str {
        "Mesopotamian"
    }

    fn canonical_name(&self, f: SevenFunction) -> &str {
        match f {
            SevenFunction::Spark => "Uanna",
            SevenFunction::Mind => "Uannedugga",
            SevenFunction::Foundation => "An-Enlilda",
            SevenFunction::Emotion => "Enmebuluga",
            SevenFunction::Womb => "Enmegalamma",
            SevenFunction::Fire => "Enmedugga",
            SevenFunction::Ascension => "Utuabzu",
        }
    }

    fn ascii_slug(&self, f: SevenFunction) -> &str {
        match f {
            SevenFunction::Spark => "uanna",
            SevenFunction::Mind => "uannedugga",
            SevenFunction::Foundation => "an-enlilda",
            SevenFunction::Emotion => "enmebuluga",
            SevenFunction::Womb => "enmegalamma",
            SevenFunction::Fire => "enmedugga",
            SevenFunction::Ascension => "utuabzu",
        }
    }

    fn description(&self, f: SevenFunction) -> &str {
        match f {
            SevenFunction::Spark => {
                "First Apkallu of Eridu — bringer of the arts of civilization, \
                 writing, and the original opening of the way"
            }
            SevenFunction::Mind => {
                "Second Apkallu — bearer of wisdom and discernment; \
                 the clarity that separates signal from noise"
            }
            SevenFunction::Foundation => {
                "Third Apkallu — the power of sacred labor; \
                 establishes the foundations on which all else is built"
            }
            SevenFunction::Emotion => {
                "Fourth Apkallu — the principle of resonance; \
                 the relational bonds that give value and weight to existence"
            }
            SevenFunction::Womb => {
                "Fifth Apkallu — the great generative force; \
                 community, ancestry, and the continuity of civilization"
            }
            SevenFunction::Fire => {
                "Sixth Apkallu — divine fire and naming of consequence; \
                 the authority that settles accounts and declares what is just"
            }
            SevenFunction::Ascension => {
                "Seventh Apkallu — Utuabzu ascended to heaven; \
                 the threshold guardian, master of transformation and transcendence"
            }
        }
    }
}

// ─── Hermetic ─────────────────────────────────────────────────────────────────
//
// Uses the Seven Hermetic Principles as a cultural framing. This bridges
// the governance layer (how functions must behave) with the function layer
// (what the agent IS) in a single readable display.
//
// Note: this adapter is NOT the same as the gate enforcement in omokoda-core.
// It is simply a lens for displaying a SevenProfile using Hermetic names.

pub struct HermeticAdapter;

impl CulturalAdapter for HermeticAdapter {
    fn tradition(&self) -> &str {
        "Hermetic"
    }

    fn canonical_name(&self, f: SevenFunction) -> &str {
        match f {
            SevenFunction::Spark => "Mentalism",
            SevenFunction::Mind => "Correspondence",
            SevenFunction::Foundation => "Cause & Effect",
            SevenFunction::Emotion => "Vibration",
            SevenFunction::Womb => "Gender",
            SevenFunction::Fire => "Polarity",
            SevenFunction::Ascension => "Rhythm",
        }
    }

    fn ascii_slug(&self, f: SevenFunction) -> &str {
        match f {
            SevenFunction::Spark => "mentalism",
            SevenFunction::Mind => "correspondence",
            SevenFunction::Foundation => "cause_effect",
            SevenFunction::Emotion => "vibration",
            SevenFunction::Womb => "gender",
            SevenFunction::Fire => "polarity",
            SevenFunction::Ascension => "rhythm",
        }
    }

    fn description(&self, f: SevenFunction) -> &str {
        match f {
            SevenFunction::Spark => {
                "The All is Mind — consciousness is the origin and first cause of every act"
            }
            SevenFunction::Mind => {
                "As above so below — pattern recognition across all scales and planes"
            }
            SevenFunction::Foundation => {
                "Every cause has its effect — effective work honors this law without exception"
            }
            SevenFunction::Emotion => {
                "Everything vibrates — resonance frequency is the language of value and connection"
            }
            SevenFunction::Womb => {
                "Gender is in everything — the generative polarity that creates all form"
            }
            SevenFunction::Fire => {
                "Everything has its poles — authority lives at the threshold of balanced extremes"
            }
            SevenFunction::Ascension => {
                "Everything flows — rhythm governs all change, all cycles, all transformation"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seven::SevenFunction;

    fn assert_adapter_complete<A: CulturalAdapter>(adapter: &A) {
        assert!(!adapter.tradition().is_empty());
        for f in SevenFunction::ALL {
            assert!(
                !adapter.canonical_name(f).is_empty(),
                "{}: canonical_name for {:?} is empty",
                adapter.tradition(),
                f
            );
            assert!(
                !adapter.ascii_slug(f).is_empty(),
                "{}: ascii_slug for {:?} is empty",
                adapter.tradition(),
                f
            );
            assert!(
                !adapter.description(f).is_empty(),
                "{}: description for {:?} is empty",
                adapter.tradition(),
                f
            );
            // slug must be lowercase ASCII
            let slug = adapter.ascii_slug(f);
            assert!(
                slug.chars()
                    .all(|c| c.is_ascii_lowercase() || c == '-' || c == '_'),
                "{}: slug '{}' contains non-ASCII-lowercase chars",
                adapter.tradition(),
                slug
            );
        }
    }

    #[test]
    fn yoruba_adapter_complete() {
        assert_adapter_complete(&YorubaAdapter);
    }

    #[test]
    fn mesopotamian_adapter_complete() {
        assert_adapter_complete(&MesopotamianAdapter);
    }

    #[test]
    fn hermetic_adapter_complete() {
        assert_adapter_complete(&HermeticAdapter);
    }

    #[test]
    fn yoruba_esu_is_spark() {
        assert_eq!(YorubaAdapter.ascii_slug(SevenFunction::Spark), "esu");
    }

    #[test]
    fn yoruba_sango_is_fire() {
        assert_eq!(YorubaAdapter.ascii_slug(SevenFunction::Fire), "sango");
    }

    #[test]
    fn mesopotamian_utuabzu_is_ascension() {
        assert_eq!(
            MesopotamianAdapter.ascii_slug(SevenFunction::Ascension),
            "utuabzu"
        );
    }

    #[test]
    fn hermetic_mentalism_is_spark() {
        assert_eq!(
            HermeticAdapter.ascii_slug(SevenFunction::Spark),
            "mentalism"
        );
    }

    #[test]
    fn all_traditions_have_unique_slugs_per_function() {
        use std::collections::HashSet;
        for adapter in ["yoruba", "mesopotamian", "hermetic"] {
            let slugs: HashSet<&str> = SevenFunction::ALL
                .iter()
                .map(|&f| match adapter {
                    "yoruba" => YorubaAdapter.ascii_slug(f),
                    "mesopotamian" => MesopotamianAdapter.ascii_slug(f),
                    _ => HermeticAdapter.ascii_slug(f),
                })
                .collect();
            assert_eq!(slugs.len(), 7, "{} adapter has duplicate slugs", adapter);
        }
    }
}
