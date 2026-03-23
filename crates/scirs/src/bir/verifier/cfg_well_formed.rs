//! BIR verifier pass: cfg_well_formed
//!
//! Every block has a terminator; terminator targets reference valid BlockIds.

use crate::bir::cfg::{Function, Terminator};
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
        match &block.term {
            Terminator::Branch { then_bb, else_bb, .. } => {
                if !valid_ids.contains(&then_bb.0) {
                    errors.push(VerifyError::new(
                        PASS,
                        format!(
                            "in {}, block {} branch target {then_bb} is not a valid block",
                            func.id, block.id
                        ),
                    ));
                }
                if !valid_ids.contains(&else_bb.0) {
                    errors.push(VerifyError::new(
                        PASS,
                        format!(
                            "in {}, block {} branch target {else_bb} is not a valid block",
                            func.id, block.id
                        ),
                    ));
                }
            }
            Terminator::Jump(target) => {
                if !valid_ids.contains(&target.0) {
                    errors.push(VerifyError::new(
                        PASS,
                        format!(
                            "in {}, block {} jump target {target} is not a valid block",
                            func.id, block.id
                        ),
                    ));
                }
            }
            Terminator::TxnExit { .. } | Terminator::Unreachable => {}
        }
    }
}
