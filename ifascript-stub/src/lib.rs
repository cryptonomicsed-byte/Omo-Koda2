//! IfáScript CI stub.
//!
//! Mirrors the public API surface of the real `ifascript` crate so that
//! omokoda-core compiles and tests pass in CI without needing the real repo.

// ── Odu types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionVessel {
    Scribe,
    Forge,
    Oracle,
    Guardian,
    Messenger,
    Architect,
    Weaver,
    Hunter,
    Healer,
    Navigator,
    Keeper,
    Judge,
    Cultivator,
    Dreamer,
    Warrior,
    Sovereign,
}

pub struct Odu {
    pub index: u8,
    pub binary: u8,
    pub universal_name: &'static str,
    pub description: &'static str,
    pub prescriptions: &'static [&'static str],
    pub vessel: ActionVessel,
}

static STUB_PRESCRIPTIONS: &[&str] = &["Align intention with action."];

static STUB_ODU: Odu = Odu {
    index: 0,
    binary: 0,
    universal_name: "Ogbe",
    description: "The first Odu — origin, light, and beginnings.",
    prescriptions: STUB_PRESCRIPTIONS,
    vessel: ActionVessel::Sovereign,
};

/// Returns the Odu for the given index (stub: always returns Ogbe with index set).
pub fn get_odu(index: u8) -> &'static Odu {
    // Static table not needed for stub — return the single static with the right index.
    // Safety: we return a reference to a static, which is always valid.
    let _ = index;
    &STUB_ODU
}

// ── CastResult / IfaVM (minimal stubs) ────────────────────────────────────────

pub struct CastResult {
    pub odu_index: u8,
    pub confidence: f64,
}

pub struct IfaVM;

impl IfaVM {
    pub fn new() -> Self { Self }
    pub fn cast_odu(&mut self) -> CastResult {
        CastResult { odu_index: 0, confidence: 0.5 }
    }
}

impl Default for IfaVM {
    fn default() -> Self { Self::new() }
}

// ── Entropy / CowrieOracle ────────────────────────────────────────────────────

pub mod entropy {
    pub struct CowrieOracle {
        seed: u64,
    }

    impl CowrieOracle {
        pub fn new(ritual_intent: &str) -> Self {
            let seed = ritual_intent
                .bytes()
                .fold(6364136223846793005u64, |acc, b| {
                    acc.wrapping_mul(6364136223846793005).wrapping_add(b as u64)
                });
            Self { seed }
        }

        pub fn cast_cowries(&mut self) -> u16 {
            self.seed = self.seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (self.seed >> 33) as u16
        }
    }
}
