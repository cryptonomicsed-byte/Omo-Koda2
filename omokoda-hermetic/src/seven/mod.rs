// omokoda-hermetic/src/seven/mod.rs
//
// Universal Seven Functions Protocol (USF-7)
//
// Culture-neutral computational substrate. Seven functions that all conscious
// and civilizational architectures must express. Cultural adapters plug in
// their own names, mythology, and symbolism without changing the ABI.
//
// Layer topology:
//   SOURCE / OCEAN      — undifferentiated potential (latent possibility space)
//   SEVEN FUNCTIONS     — this module (universal computational ABI)
//   CULTURAL ADAPTERS   — Yorùbá, Mesopotamian, Hermetic, and future traditions
//   HERMETIC GOVERNANCE — how the functions must behave (omokoda-core/gates/)
//   REALIZATION         — receipts, reputation, physical-world action

pub mod adapters;

use crate::HermeticState;
use serde::{Deserialize, Serialize};

/// The seven universal functions of conscious and civilizational agency.
///
/// These are not gods, archetypes, or psychological types. They are
/// computational primitives — the minimum set of functional dimensions
/// required for a sovereign agent to act in the world.
///
/// Each function maps positionally to one of the Odù-derived HermeticState
/// values. Same seed, same bytes, two complementary framings:
///   HermeticPrinciple  →  how the function must behave (governance)
///   SevenFunction      →  what the function IS (ontology)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum SevenFunction {
    /// Agency · Communication · Choice · Initiation
    /// The first mover — opens the gate, routes the signal, names the choice.
    Spark = 0,

    /// Reason · Clarity · Ethics · Coherence
    /// The clarity principle — perceives pattern, refuses distortion.
    Mind = 1,

    /// Will · Labor · Execution · Embodiment
    /// The principle of effective action — brings intention into physical form.
    Foundation = 2,

    /// Value · Relationship · Resonance · Desire
    /// The connection principle — weights what matters and to whom.
    Emotion = 3,

    /// Creation · Community · Ancestry · Continuity
    /// The generative principle — brings forth, sustains, and remembers.
    Womb = 4,

    /// Power · Authority · Justice · Consequence
    /// The settlement principle — names what is owed and enforces it.
    Fire = 5,

    /// Change · Transition · Adaptation · Transformation
    /// The flow principle — navigates the threshold between states.
    Ascension = 6,
}

impl SevenFunction {
    pub const ALL: [SevenFunction; 7] = [
        Self::Spark,
        Self::Mind,
        Self::Foundation,
        Self::Emotion,
        Self::Womb,
        Self::Fire,
        Self::Ascension,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(i: usize) -> Option<Self> {
        match i {
            0 => Some(Self::Spark),
            1 => Some(Self::Mind),
            2 => Some(Self::Foundation),
            3 => Some(Self::Emotion),
            4 => Some(Self::Womb),
            5 => Some(Self::Fire),
            6 => Some(Self::Ascension),
            _ => None,
        }
    }

    /// Stable English name — consistent across all cultural adapters.
    pub fn universal_name(self) -> &'static str {
        match self {
            Self::Spark => "Spark",
            Self::Mind => "Mind",
            Self::Foundation => "Foundation",
            Self::Emotion => "Emotion",
            Self::Womb => "Womb",
            Self::Fire => "Fire",
            Self::Ascension => "Ascension",
        }
    }

    /// Core capability keywords for this function.
    pub fn capabilities(self) -> &'static [&'static str] {
        match self {
            Self::Spark => &["agency", "communication", "choice", "initiation"],
            Self::Mind => &["reason", "clarity", "ethics", "coherence"],
            Self::Foundation => &["will", "labor", "execution", "embodiment"],
            Self::Emotion => &["value", "relationship", "resonance", "desire"],
            Self::Womb => &["creation", "community", "ancestry", "continuity"],
            Self::Fire => &["power", "authority", "justice", "consequence"],
            Self::Ascension => &["change", "transition", "adaptation", "transformation"],
        }
    }
}

/// Maps the seven functions to a cultural tradition's names and descriptions.
///
/// A tradition cannot redefine the underlying function. It provides its own
/// canonical name, ASCII slug (for wire/storage keys), and one-line description
/// in the tradition's framing. The computation remains stable.
pub trait CulturalAdapter: Send + Sync {
    /// Human-readable tradition name (e.g. "Yorùbá", "Mesopotamian").
    fn tradition(&self) -> &str;

    /// Canonical name within the tradition — may include Unicode/diacritics.
    fn canonical_name(&self, f: SevenFunction) -> &str;

    /// ASCII-safe slug for storage keys and protocol wire format.
    fn ascii_slug(&self, f: SevenFunction) -> &str;

    /// One-line description framing this function through the tradition's lens.
    fn description(&self, f: SevenFunction) -> &str;
}

/// Per-agent strength profile across the seven functions.
///
/// Derived deterministically from the agent's Odù-seeded `HermeticState`.
/// The 7 HKDF-derived floats are shared between `HermeticState` (governance
/// framing) and `SevenProfile` (functional-strength framing) — same bytes,
/// different semantic lens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SevenProfile {
    values: [f64; 7],
}

impl SevenProfile {
    /// Derive from an agent's Odù-seeded HermeticState.
    ///
    /// Positional mapping (SevenFunction index → HermeticState getter):
    ///   Spark(0)      ← mentalism()      — consciousness as the initiating act
    ///   Mind(1)       ← correspondence() — pattern recognition as rational clarity
    ///   Foundation(2) ← cause_effect()   — causality as the root of effective work
    ///   Emotion(3)    ← vibration()      — resonance frequency as emotional weight
    ///   Womb(4)       ← gender()         — generative polarity as creative force
    ///   Fire(5)       ← polarity()       — extremes of power as authority/justice
    ///   Ascension(6)  ← rhythm()         — tidal flow as transformation/change
    pub fn from_hermetic(state: &HermeticState) -> Self {
        Self {
            values: [
                state.mentalism(),
                state.correspondence(),
                state.cause_effect(),
                state.vibration(),
                state.gender(),
                state.polarity(),
                state.rhythm(),
            ],
        }
    }

    pub fn strength(&self, f: SevenFunction) -> f64 {
        self.values[f.index()]
    }

    /// The function this agent is most strongly aligned with.
    pub fn dominant(&self) -> SevenFunction {
        let (idx, _) = self
            .values
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap();
        SevenFunction::from_index(idx).unwrap()
    }

    /// Composite strength: mean across all seven functions.
    pub fn composite(&self) -> f64 {
        self.values.iter().sum::<f64>() / 7.0
    }

    /// Render this profile through a cultural adapter's lens.
    pub fn display<A: CulturalAdapter>(&self, adapter: &A) -> SevenProfileDisplay {
        SevenProfileDisplay {
            tradition: adapter.tradition().to_string(),
            entries: SevenFunction::ALL
                .iter()
                .map(|&f| SevenProfileEntry {
                    function: f,
                    canonical_name: adapter.canonical_name(f).to_string(),
                    ascii_slug: adapter.ascii_slug(f).to_string(),
                    strength: self.strength(f),
                })
                .collect(),
        }
    }
}

/// Human-readable profile rendered through a specific cultural adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SevenProfileDisplay {
    pub tradition: String,
    pub entries: Vec<SevenProfileEntry>,
}

impl SevenProfileDisplay {
    /// Dominant entry (highest strength).
    pub fn dominant(&self) -> Option<&SevenProfileEntry> {
        self.entries
            .iter()
            .max_by(|a, b| a.strength.partial_cmp(&b.strength).unwrap())
    }
}

/// One entry in a `SevenProfileDisplay`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SevenProfileEntry {
    pub function: SevenFunction,
    pub canonical_name: String,
    pub ascii_slug: String,
    pub strength: f64,
}

/// Seven-day sacred calendar — each day governed by one function.
///
/// Encodes the cyclical expression of the Seven Functions through time.
/// The Yorùbá mapping (Sun=Èṣù, Mon=Ṣàngó…) is the reference implementation;
/// the universal calendar uses the same positional order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SevenCalendar;

impl SevenCalendar {
    /// Which function governs a given weekday (0 = Sunday … 6 = Saturday).
    pub fn function_for_day(day: u8) -> SevenFunction {
        match day % 7 {
            0 => SevenFunction::Spark,      // Sunday   — initiation, new beginnings
            1 => SevenFunction::Fire,       // Monday   — authority and consequence
            2 => SevenFunction::Emotion,    // Tuesday  — value and relationship
            3 => SevenFunction::Womb,       // Wednesday — creation and community
            4 => SevenFunction::Ascension,  // Thursday  — change and transition
            5 => SevenFunction::Foundation, // Friday   — work and execution
            _ => SevenFunction::Mind,       // Saturday — clarity and ethics
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HermeticState;

    #[test]
    fn all_returns_seven() {
        assert_eq!(SevenFunction::ALL.len(), 7);
    }

    #[test]
    fn roundtrip_index() {
        for f in SevenFunction::ALL {
            assert_eq!(SevenFunction::from_index(f.index()), Some(f));
        }
    }

    #[test]
    fn from_index_out_of_bounds_returns_none() {
        assert!(SevenFunction::from_index(7).is_none());
        assert!(SevenFunction::from_index(100).is_none());
    }

    #[test]
    fn universal_names_non_empty() {
        for f in SevenFunction::ALL {
            assert!(!f.universal_name().is_empty());
        }
    }

    #[test]
    fn capabilities_non_empty() {
        for f in SevenFunction::ALL {
            assert!(!f.capabilities().is_empty());
        }
    }

    #[test]
    fn profile_from_hermetic_has_seven_values() {
        let state = HermeticState::from_seed("test", 0);
        let profile = SevenProfile::from_hermetic(&state);
        for f in SevenFunction::ALL {
            let v = profile.strength(f);
            assert!((0.0..=1.0).contains(&v), "strength out of range: {}", v);
        }
    }

    #[test]
    fn profile_dominant_is_valid_function() {
        let state = HermeticState::from_seed("test", 0);
        let profile = SevenProfile::from_hermetic(&state);
        let dom = profile.dominant();
        assert!(SevenFunction::ALL.contains(&dom));
    }

    #[test]
    fn profile_composite_in_unit_range() {
        let state = HermeticState::from_seed("test", 0);
        let profile = SevenProfile::from_hermetic(&state);
        let c = profile.composite();
        assert!((0.0..=1.0).contains(&c));
    }

    #[test]
    fn calendar_covers_all_seven_functions() {
        use std::collections::HashSet;
        let covered: HashSet<_> = (0u8..7).map(SevenCalendar::function_for_day).collect();
        assert_eq!(covered.len(), 7, "calendar must cover all 7 functions");
    }
}
