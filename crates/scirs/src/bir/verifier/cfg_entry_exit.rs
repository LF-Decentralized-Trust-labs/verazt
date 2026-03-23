//! BIR verifier pass: cfg_entry_exit
//!
//! Exactly one entry block (%bb0); every TxnExit block has no successors.

use crate::bir::cfg::{Function, Terminator};
use crate::bir::module::Module;
use crate::verify::VerifyError;

const PASS: &str = "bir::cfg_entry_exit";

pub fn check(module: &Module) -> Vec<VerifyError> {
    let mut errors = Vec::new();

    for func in &module.functions {
        check_function(func, &mut errors);
    }

    errors
}

fn check_function(func: &Function, errors: &mut Vec<VerifyError>) {
    // Check entry block is %bb0
    if let Some(first) = func.blocks.first() {
        if first.id.0 != 0 {
            errors.push(VerifyError::new(
                PASS,
                format!(
                    "in {}, first block is {} but expected %bb0",
                    func.id, first.id
                ),
            ));
        }
    } else {
        errors.push(VerifyError::new(
            PASS,
            format!("in {}, function has no blocks", func.id),
        ));
    }

    // Check TxnExit blocks have no successors (this is structural:
    // TxnExit variant has no target fields, so it always has 0 successors).
    // We still verify that only terminal blocks use TxnExit, by noting
    // there's nothing else to check here structurally.
    // Additionally, check that there's at least one exit (TxnExit or Unreachable).
    let has_exit = func.blocks.iter().any(|b| {
        matches!(
            b.term,
            Terminator::TxnExit { .. } | Terminator::Unreachable
        )
    });

    if !has_exit && !func.blocks.is_empty() {
        errors.push(VerifyError::new(
            PASS,
            format!(
                "in {}, no exit block (TxnExit or Unreachable) found",
                func.id
            ),
        ));
    }
}
