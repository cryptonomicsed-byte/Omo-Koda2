# Phase 2: Hermetic Principle Engine (✅ done)
## Directory: omokoda-hermetic/
- Cargo.toml – includes `serde` and `chrono`
- src/lib.rs – implements:
  - `risk_score(prompt: &str) -> f32` (Mentalism, Correspondence)
  - `polarity_score(tool: &str, args: &str) -> f32` (Polarity)
  - `rhythm_check(cooldown: i64, elapsed: i64) -> bool`
  - `gender_balance(think:u32, act:u32) -> f32`
  - `apply_vibration_decay(agent: &mut Agent)`
All functions are pure and unit‑tested (see `tests.rs`).
