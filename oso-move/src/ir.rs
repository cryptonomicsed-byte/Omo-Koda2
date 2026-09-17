//! OSO-IR types — mirrored from oso-parser/src/ir.rs and oso-ir-spec.md.
//!
//! These types represent the JSON-serialisable intermediate representation
//! that bridges Ọ̀ṢỌ́ source → Move / WASM / Native backends.
//!
//! We mirror rather than depend on oso-parser to avoid cross-crate coupling
//! during the standalone oso-move binary phase.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// OSO-IR top-level schema (matches oso-ir-spec.md)
// ---------------------------------------------------------------------------

/// Top-level OSO-IR document.
///
/// Fields match the JSON schema defined in specs/oso-ir-spec.md.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsoIR {
    /// Must be "1.0".
    pub oso_ir_version: String,

    /// One of: financial, agent, work, device, evidence, governance.
    pub contract_class: String,

    /// PascalCase contract name. Becomes the Move module name.
    pub contract_name: String,

    /// Asset structs declared by this contract.
    pub assets: Vec<AssetSpec>,

    /// Capability tokens this contract consumes or grants.
    #[serde(default)]
    pub capabilities: Vec<String>,

    /// Minimum agent tier (0–5).
    #[serde(default)]
    pub minimum_tier: u8,

    /// Evidence requirement.
    pub evidence: EvidenceSpec,

    /// Witnessing policy (optional).
    #[serde(default)]
    pub witness_policy: Option<WitnessPolicy>,

    /// Settlement parameters.
    pub settlement: SettlementSpec,

    /// Freeform policy map.
    pub policy: HashMap<String, serde_json::Value>,

    /// Ordered state-machine labels. First = initial; last = terminal.
    pub lifecycle: Vec<String>,
}

/// One asset (struct) declared in the contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetSpec {
    /// PascalCase struct name.
    pub name: String,
    /// snake_case field names.
    pub fields: Vec<String>,
}

/// Evidence requirement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSpec {
    pub required: bool,
    #[serde(rename = "type", default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub fields: Vec<String>,
}

/// Witness / quorum policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessPolicy {
    pub quorum: u8,
    pub types: Vec<String>,
}

/// Settlement parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementSpec {
    pub currency: String,
    pub fee_routing: String,
    #[serde(default)]
    pub creator_share: f64,
    #[serde(default)]
    pub burn_share: f64,
    #[serde(default)]
    pub provider_share: f64,
}

// ---------------------------------------------------------------------------
// VM-level IR (mirrors oso-parser IrInstruction for programs-as-instructions)
// ---------------------------------------------------------------------------

/// A single VM-level instruction (opcode + named args).
/// Used when a program.json contains instructions rather than a contract schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrInstruction {
    pub opcode: u8,
    pub opcode_name: String,
    #[serde(default)]
    pub args: HashMap<String, IrValue>,
    #[serde(default)]
    pub line: usize,
}

/// Argument value types in Ọ̀ṢỌ́ IR.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IrValue {
    Int(i64),
    Float(f64),
    Str(String),
    Hex(u64),
    Block(Vec<IrInstruction>),
}

/// An Ọ̀ṢỌ́ program as a flat list of VM instructions.
pub type IrProgram = Vec<IrInstruction>;

// ---------------------------------------------------------------------------
// OpCode enum — human-readable aliases over opcode bytes
// ---------------------------------------------------------------------------

/// Named opcode variants relevant to Move codegen.
/// Other opcodes compile to native comments.
#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {
    /// 0x50 CALL — cross-module agent call
    AgentCall,
    /// 0x72 TRANSFER_ASE — Àṣẹ transfer → sui::transfer stub
    Transfer,
    /// Emit a Zàngbétò receipt event — maps to Sui event emission
    EmitReceipt,
    /// Any opcode that has no Move equivalent — compiled as a comment
    Native(String),
}

impl OpCode {
    /// Classify an opcode byte or opcode_name string.
    pub fn from_instruction(instr: &IrInstruction) -> Self {
        match instr.opcode {
            0x50 => OpCode::AgentCall,
            0x72 => OpCode::Transfer,
            // EMIT_RECEIPT is not a single opcode in the VM table;
            // we detect it by opcode_name convention.
            _ => {
                let name = instr.opcode_name.to_ascii_uppercase();
                match name.as_str() {
                    "AGENT_CALL" | "CALL" => OpCode::AgentCall,
                    "TRANSFER" | "TRANSFER_ASE" => OpCode::Transfer,
                    "EMIT_RECEIPT" | "EMIT_ASE" => OpCode::EmitReceipt,
                    other => OpCode::Native(other.to_string()),
                }
            }
        }
    }
}
