//! BIR verifier pass: op_id_unique
//!
//! All OpId values are unique within a function.

use crate::bir::cfg::Function;
use crate::bir::module::Module;
use crate::verify::VerifyError;
use std::collections::HashSet;

const PASS: &str = "bir::op_id_unique";

pub fn check(module: &Module) -> Vec<VerifyError> {
    let mut errors = Vec::new();

    for func in &module.functions {
        check_function(func, &mut errors);
    }

    errors
}

fn check_function(func: &Function, errors: &mut Vec<VerifyError>) {
    let mut seen: HashSet<usize> = HashSet::new();

    for block in &func.blocks {
        for op in &block.ops {
            if !seen.insert(op.id.0) {
                let mut err = VerifyError::new(
                    PASS,
                    format!("in {}, duplicate OpId {} in block {}", func.id, op.id, block.id),
                );
                if let Some(span) = op.span {
                    err = err.with_span(span);
                }
                errors.push(err);
            }
        }
    }
}
