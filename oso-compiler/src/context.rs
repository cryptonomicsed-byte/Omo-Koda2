//! CompileContext — values injected as @LOAD_CONST preamble into every program.

use serde::{Deserialize, Serialize};

/// Values available to a Ọ̀ṢỌ́ program at compile time.
/// These are prepended as `@LOAD_CONST` instructions before the ritual source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileContext {
    /// Aggregate hermetic gate alignment (0.0..1.0).
    pub gate_alignment: f32,
    /// Odù tile index active for this execution.
    pub odu_index: u8,
    /// Àṣẹ emission multiplier from gate evaluation.
    pub ase_multiplier: f32,
    /// Caller-provided params (forwarded as JSON).
    pub params: serde_json::Value,
}

impl CompileContext {
    /// Build a preamble Ọ̀ṢỌ́ source that injects context as constants.
    ///
    /// The generated source is prepended to the ritual source before parsing.
    /// Each constant is named `__ctx_<key>__` to avoid collision with user vars.
    pub fn to_preamble_source(&self) -> String {
        format!(
            "@LOAD_CONST(key=\"__ctx_gate_alignment__\", value={:.6})\n\
             @LOAD_CONST(key=\"__ctx_odu_index__\", value={})\n\
             @LOAD_CONST(key=\"__ctx_ase_multiplier__\", value={:.6})\n",
            self.gate_alignment, self.odu_index, self.ase_multiplier,
        )
    }
}
