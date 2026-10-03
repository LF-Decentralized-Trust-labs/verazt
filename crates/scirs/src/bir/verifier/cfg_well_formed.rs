//! BIR verifier pass: cfg_well_formed
//!
//! Every block has a terminator; terminator targets reference valid BlockIds.

use crate::bir::cfg::Function;
use crate::bir::module::Module;
use crate::verify::VerifyError;
use std::collections::HashSet;

const PASS: &str = "bir::cfg_well_formed";

pub fn check(module: &Module) -> Vec<VerifyError> {
    let mut errors = Vec::new();

    for func in &module.functions {
        check_function(func, &mut errors);
    }

    errors
}

fn check_function(func: &Function, errors: &mut Vec<VerifyError>) {
    let valid_ids: HashSet<usize> = func.blocks.iter().map(|b| b.id.0).collect();

    for block in &func.blocks {
        for target in block.term.successors() {
            if !valid_ids.contains(&target.0) {
                errors.push(VerifyError::new(
                    PASS,
                    format!(
                        "in {}, block {} transfers to {target}, which is not a valid block",
                        func.id, block.id
                    ),
                ));
            }
        }
    }
}
