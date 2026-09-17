//! Resource rules:
//!   RES1 — warn if the program has > 10 TRANSFER/ESCROW/BURN instructions
//!           combined (high cost estimate)
//!   RES2 — warn if total estimated ASE cost (from `amount` args) exceeds 10,000

use crate::{Diagnostic, IrInstruction};

/// Economy opcodes that carry real ASE cost.
const COSTLY_OPCODES: &[&str] = &["TRANSFER_ASE", "LOCK_ASE", "BURN_ASE", "EMIT_ASE"];

/// Threshold above which we warn about instruction count.
const COSTLY_COUNT_THRESHOLD: usize = 10;

/// Threshold above which we warn about total ASE amount.
const COSTLY_ASE_THRESHOLD: u64 = 10_000;

pub fn check(
    instructions: &[IrInstruction],
    warnings: &mut Vec<Diagnostic>,
    _errors: &mut Vec<Diagnostic>,
) {
    let mut costly_count: usize = 0;
    let mut total_ase: u64 = 0;

    for instr in instructions {
        if COSTLY_OPCODES.contains(&instr.opcode_name.as_str()) {
            costly_count += 1;

            // Sum up `amount` arg if present
            if let Some(amount_val) = instr.args.get("amount") {
                let amount = match amount_val {
                    serde_json::Value::Number(n) => n.as_u64().unwrap_or(0),
                    _ => 0,
                };
                total_ase = total_ase.saturating_add(amount);
            }
        }
    }

    if costly_count > COSTLY_COUNT_THRESHOLD {
        warnings.push(Diagnostic {
            rule: "resource.RES1.high_cost_instruction_count".into(),
            message: format!(
                "program contains {} costly economy instructions (TRANSFER/LOCK/BURN/EMIT), \
                 which exceeds the threshold of {}. Consider batching or splitting.",
                costly_count, COSTLY_COUNT_THRESHOLD
            ),
            line: None,
        });
    }

    if total_ase > COSTLY_ASE_THRESHOLD {
        warnings.push(Diagnostic {
            rule: "resource.RES2.high_ase_cost".into(),
            message: format!(
                "estimated total ASE movement is {} which exceeds threshold of {}. \
                 Verify this is intentional.",
                total_ase, COSTLY_ASE_THRESHOLD
            ),
            line: None,
        });
    }
}
