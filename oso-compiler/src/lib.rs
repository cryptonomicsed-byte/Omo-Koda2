//! Ọ̀ṢỌ́ compiler — bridges If-Script decisions to OSOVM IR.
//!
//! See spec: sovereign-eco-blueprint/specs/OSO_IFSCRIPT_BOUNDARY.md
//!
//! Pipeline:
//!   IfDecision → oso_dispatch() → IrProgram
//!
//! The compiler layers context (gate_alignment, params) onto the Ọ̀ṢỌ́ source
//! before parsing, injecting them as `@LOAD_CONST` preamble instructions.

pub mod decision;
pub mod context;
pub mod dispatch;
pub mod registry;

pub use decision::{IfDecision, GateScore};
pub use context::CompileContext;
pub use dispatch::oso_dispatch;
pub use registry::RitualRegistry;
pub use oso_parser::{IrProgram, IrInstruction, IrValue, OsoError, OsoResult};
