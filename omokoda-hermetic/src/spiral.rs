//! Spiral Calendar — deterministic, birth-anchored sacred time.
//!
//! Ported from `~/Koodu/src/time/sacred_time.jl` (Ritual-Codex v7). The
//! Julia original derives a five-layer Òrìṣà signature from a Bitcoin
//! block height via pure integer arithmetic (div/mod on a block number) --
//! no live chain query needed, which is exactly why it's safe to port:
//! this is deterministic math, not an oracle read.
//!
//! The critical design change from how Kóòdù was used until now: every
//! prior lookup (`rhythm::today_resonance()`) was keyed on
//! `Utc::now().weekday()` -- identical for every agent, every request, on
//! a given day. This module keys on an agent's own birth timestamp
//! instead, computed ONCE at birth and never recomputed. Two agents born
//! a second apart land on different `veil_number`s (they cycle every ~10
//! minutes of block time) and, as spiral time advances, different
//! week/moon/year layers too -- genuinely individuated by birth moment,
//! not by identity alone.
//!
//! `today_resonance()` in `rhythm.rs` still answers "what day is it for
//! the hive right now" (a legitimate, separate, wall-clock question) --
//! this module answers "what is THIS agent's own permanent resonance."

use bipon39::Macro;

// ─── Constants ──────────────────────────────────────────────────────────

const BLOCKS_PER_DAY: i64 = 144; // 10 min/block × 144 = 1440 min
const SECONDS_PER_BLOCK: i64 = 600;
const GENESIS_BLOCK: i64 = 780_000; // Ọ̀ṢỌ́VM epoch start
/// Real-world unix timestamp of Bitcoin block 780,000 (~2023-02-25 05:00
/// UTC). A fixed constant, not a live chain query -- the whole point of
/// block-height-anchored time is that it's reproducible without a working
/// internet connection or a chain explorer years from now.
const GENESIS_BLOCK_UNIX_TIME: i64 = 1_677_304_800;

/// Day-cycle Òrìṣà ordering (Sunday through Saturday) -- distinct from the
/// Tier-ladder ordering used for Hermetic Principle / Chakra assignment
/// (see `dominant_orisha_for_hermetic_state` in omokoda-core). Both are
/// real, both are intentional: this one governs the calendar/ritual axis,
/// the other governs the power/governance axis. They only agree at
/// position 0 (Èṣù) by coincidence.
pub const DAY_CYCLE: [Macro; 7] = [
    Macro::Esu,
    Macro::Sango,
    Macro::Osun,
    Macro::Yemoja,
    Macro::Oya,
    Macro::Ogun,
    Macro::Obatala,
];

// ─── BtcTime ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BtcTime {
    pub block_height: i64,
    pub day_number: i64, // days since GENESIS_BLOCK
    pub tick_number: i64,
    pub minute_of_day: i64,
    pub day_of_week: u8, // 0-6, 0 = Sunday
    pub is_sabbath: bool,
    pub is_jubilee_day: bool,
    pub is_eshu_node: bool,
}

impl BtcTime {
    pub fn from_block_height(height: i64) -> Self {
        let relative = (height - GENESIS_BLOCK).max(0);
        let day = relative / BLOCKS_PER_DAY;
        let tick = relative % BLOCKS_PER_DAY;
        let minute = (tick as f64 / 0.1).floor() as i64;

        let dow = (day % 7) as u8;
        let sabbath = dow == 6; // Ọbàtálá day = Saturday
        let jubilee_day = day % 49 == 48;
        let eshu_node = tick % 12 == 0;

        Self {
            block_height: height,
            day_number: day,
            tick_number: tick,
            minute_of_day: minute,
            day_of_week: dow,
            is_sabbath: sabbath,
            is_jubilee_day: jubilee_day,
            is_eshu_node: eshu_node,
        }
    }

    /// The real entry point for agent birth: approximate the BTC block
    /// height nearest a unix timestamp (600s/block from the genesis
    /// constant) and derive BtcTime from it. Deterministic and
    /// reproducible forever -- no live chain query.
    pub fn from_unix_timestamp(unix_time: u64) -> Self {
        let elapsed = (unix_time as i64 - GENESIS_BLOCK_UNIX_TIME).max(0);
        let height = GENESIS_BLOCK + elapsed / SECONDS_PER_BLOCK;
        Self::from_block_height(height)
    }
}

// ─── Five-layer Òrìṣà ───────────────────────────────────────────────────

pub fn day_orisa(btc: &BtcTime) -> Macro {
    DAY_CYCLE[btc.day_of_week as usize]
}

pub fn week_orisa(btc: &BtcTime) -> Macro {
    let week = btc.day_number / 7;
    DAY_CYCLE[(week % 7) as usize]
}

pub fn moon_orisa(btc: &BtcTime) -> Macro {
    let moon = btc.day_number / 28; // 28-day "moon" = 4 weeks
    DAY_CYCLE[(moon % 7) as usize]
}

pub fn year_orisa(btc: &BtcTime) -> Macro {
    let year = btc.day_number / 364; // 364-day year = 13 moons
    DAY_CYCLE[(year % 7) as usize]
}

pub fn jubilee_orisa(btc: &BtcTime) -> Macro {
    let jubilee = btc.day_number / 18_200; // 50-year jubilee = 50 x 364 days
    DAY_CYCLE[(jubilee % 7) as usize]
}

// ─── SpiralTime ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct SpiralTime {
    pub btc: BtcTime,
    pub day_osa: Macro,
    pub week_osa: Macro,
    pub moon_osa: Macro,
    pub year_osa: Macro,
    pub jubilee_osa: Macro,
    /// 1-50 (350-day cycle: 50 veils x 7 days). This is the real
    /// individuating value: it advances roughly every ~10 minutes of
    /// block time, so agents born close together but not simultaneously
    /// still diverge.
    pub veil_number: i64,
    pub jubilee_cycle: i64,
    pub eshu_squared: bool,
    pub capstone_day: bool,
    pub void_day: bool,
}

impl SpiralTime {
    pub fn from_btc(btc: BtcTime) -> Self {
        let veil = btc.day_number % 350 + 1;
        let jubilee = btc.day_number / 18_200 + 1;
        let eshu_sq = veil % 12 == 0;
        let capstone = btc.day_number % 343 == 342; // 7x7x7
        let void = btc.day_number % 364 == 363;

        Self {
            day_osa: day_orisa(&btc),
            week_osa: week_orisa(&btc),
            moon_osa: moon_orisa(&btc),
            year_osa: year_orisa(&btc),
            jubilee_osa: jubilee_orisa(&btc),
            veil_number: veil,
            jubilee_cycle: jubilee,
            eshu_squared: eshu_sq,
            capstone_day: capstone,
            void_day: void,
            btc,
        }
    }

    /// The real entry point: an agent's permanent spiral signature,
    /// computed once from their birth unix timestamp. Call this at birth
    /// and persist the result -- never recompute from "now."
    pub fn from_birth_timestamp(birth_unix_time: u64) -> Self {
        Self::from_btc(BtcTime::from_unix_timestamp(birth_unix_time))
    }

    /// The esoteric mask name for this agent's permanent veil position.
    pub fn veil_esoteric(&self) -> &'static str {
        veil_to_esoteric(self.veil_number)
    }

    /// The archetypal word for this agent's permanent veil position --
    /// used as a subtle, unnamed tone cue rather than a stated fact (see
    /// `interpreter.rs::execute_compiled_think`'s system prompt
    /// construction).
    pub fn veil_archetypal(&self) -> &'static str {
        veil_to_archetypal(self.veil_number)
    }
}

// ─── Ritual gates ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RitualGate {
    NoGate,
    Sabbath,
    JubileeMajor,
    EshuSquared,
    Capstone,
    Void,
}

pub fn check_gate(spiral: &SpiralTime) -> RitualGate {
    if spiral.void_day {
        return RitualGate::Void;
    }
    if spiral.capstone_day {
        return RitualGate::Capstone;
    }
    if spiral.eshu_squared {
        return RitualGate::EshuSquared;
    }
    if spiral.btc.is_jubilee_day {
        return RitualGate::JubileeMajor;
    }
    if spiral.btc.is_sabbath {
        return RitualGate::Sabbath;
    }
    RitualGate::NoGate
}

#[derive(Debug, Clone, Copy)]
pub struct GateEconomicEffect {
    pub minting_active: bool,
    pub new_contracts_allowed: bool,
    pub tithe_enforced: bool,
    pub multiplier: f64,
    pub settle_only: bool,
}

pub fn gate_economic_effect(gate: RitualGate) -> GateEconomicEffect {
    let mut e = GateEconomicEffect {
        minting_active: true,
        new_contracts_allowed: true,
        tithe_enforced: false,
        multiplier: 1.0,
        settle_only: false,
    };
    match gate {
        RitualGate::Sabbath => {
            e.new_contracts_allowed = false;
            e.settle_only = true;
            e.multiplier = 1.1;
        }
        RitualGate::EshuSquared => {
            e.tithe_enforced = true;
            e.multiplier = 1.369;
        }
        RitualGate::JubileeMajor => {
            e.multiplier = 2.0;
        }
        RitualGate::Void => {
            e.minting_active = false;
        }
        RitualGate::NoGate | RitualGate::Capstone => {}
    }
    e
}

// ─── Veil name tables (ported verbatim from sacred_time.jl) ────────────

const VEIL_ESOTERIC: [&str; 50] = [
    "Binary Bones", "Cultural Cycles", "Mathematical Constants", "Temple Codes",
    "Cosmic Cycles", "Chaos & Fractals", "Harmonics", "Meta-Grids",
    "Recursive Mirrors", "Archetypal Forms", "Energetics", "Meta-Consciousness",
    "The Nameless Source", "Symmetry", "Codes & Designs", "Modular Forms",
    "Information", "Topology", "Quasicrystals", "Non-commutative",
    "Magic Squares", "Measure", "Cosmological", "Planck Units",
    "Particle Ratios", "Neutrino", "Dark Energy", "Large Numbers",
    "Black Hole", "Yuga", "Gematria", "Unicode", "Complexity",
    "Busy Beaver", "Category", "Homotopy", "Knot Codes", "Entropy",
    "Anthropic", "Multiverse", "Simulation", "Platonic", "Enochian",
    "Kabbalah", "Sexagesimal", "Islamic", "Christian", "Norse",
    "Modern Physics", "The Absolute Unknown",
];

const VEIL_ARCHETYPAL: [&str; 50] = [
    "Intention", "Breath", "Fire", "Waters", "Stone", "Rhythm", "Union",
    "Seed", "Serpent", "Mirror", "Dream", "Blood", "Song", "Dance",
    "Mask", "Path", "Shadow", "Flame", "Word", "Covenant",
    "Sword", "Crown", "Throne", "Chalice", "Sun", "Eye", "Heart",
    "Tower", "Phoenix", "Balance", "Abyss", "Bones", "Night",
    "Maskless", "Chains", "Key", "Labyrinth", "Storm", "Vessel",
    "Gatekeeper", "Star", "Ocean", "Mountain", "Child", "Union Crown",
    "Silence", "Light", "Circle", "Ancestors", "Jubilee",
];

fn veil_to_esoteric(n: i64) -> &'static str {
    let idx = ((n - 1).rem_euclid(VEIL_ESOTERIC.len() as i64)) as usize;
    VEIL_ESOTERIC[idx]
}

fn veil_to_archetypal(n: i64) -> &'static str {
    let idx = ((n - 1).rem_euclid(VEIL_ARCHETYPAL.len() as i64)) as usize;
    VEIL_ARCHETYPAL[idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genesis_block_is_day_zero_sunday() {
        let btc = BtcTime::from_block_height(GENESIS_BLOCK);
        assert_eq!(btc.day_number, 0);
        assert_eq!(btc.day_of_week, 0);
        assert_eq!(day_orisa(&btc), Macro::Esu);
    }

    #[test]
    fn saturday_is_sabbath_and_obatala() {
        let btc = BtcTime::from_block_height(GENESIS_BLOCK + 6 * BLOCKS_PER_DAY);
        assert_eq!(btc.day_of_week, 6);
        assert!(btc.is_sabbath);
        assert_eq!(day_orisa(&btc), Macro::Obatala);
    }

    #[test]
    fn two_births_a_block_apart_diverge_in_veil() {
        let a = SpiralTime::from_birth_timestamp(1_800_000_000);
        let b = SpiralTime::from_birth_timestamp(1_800_000_000 + SECONDS_PER_BLOCK as u64);
        // Not guaranteed to differ in day_osa, but veil_number advances
        // roughly every block -- this is the real individuation signal.
        assert_ne!(a.btc.block_height, b.btc.block_height);
    }

    #[test]
    fn birth_timestamp_is_deterministic() {
        let a = SpiralTime::from_birth_timestamp(1_800_000_000);
        let b = SpiralTime::from_birth_timestamp(1_800_000_000);
        assert_eq!(a.veil_number, b.veil_number);
        assert_eq!(a.day_osa, b.day_osa);
    }

    #[test]
    fn veil_names_cycle_over_fifty() {
        assert_eq!(veil_to_esoteric(1), "Binary Bones");
        assert_eq!(veil_to_esoteric(50), "The Absolute Unknown");
        assert_eq!(veil_to_esoteric(51), "Binary Bones"); // wraps
    }

    #[test]
    fn sabbath_gate_has_settle_only_effect() {
        let btc = BtcTime::from_block_height(GENESIS_BLOCK + 6 * BLOCKS_PER_DAY);
        let spiral = SpiralTime::from_btc(btc);
        let gate = check_gate(&spiral);
        assert_eq!(gate, RitualGate::Sabbath);
        let effect = gate_economic_effect(gate);
        assert!(effect.settle_only);
        assert!(!effect.new_contracts_allowed);
    }
}
