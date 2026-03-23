//! FIR verifier pass: param_arity
//!
//! Tail call argument count matches the callee function's parameter count.

use crate::fir::ops::{Module, TailCallData, Terminator};
use crate::verify::VerifyError;
use std::collections::HashMap;

const PASS: &str = "fir::param_arity";

pub fn check(module: &Module) -> Vec<VerifyError> {
    let mut errors = Vec::new();

    // Build map: function name -> param count
    let param_counts: HashMap<&str, usize> = module
        .functions
        .iter()
        .map(|f| (f.id.0.as_str(), f.params.len()))
        .collect();

    for func in &module.functions {
        check_terminator(&func.term, &param_counts, &mut errors);
    }

    errors
}

fn check_tail_call(
    tc: &TailCallData,
    param_counts: &HashMap<&str, usize>,
    errors: &mut Vec<VerifyError>,
) {
    if let Some(&expected) = param_counts.get(tc.callee.0.as_str()) {
        let actual = tc.args.len();
        if actual != expected {
            errors.push(VerifyError::new(
                PASS,
                format!(
                    "tail call to {} has {actual} args but callee expects {expected} params",
                    tc.callee
                ),
            ));
        }
    }
    // If callee not found, tail_call_targets pass already reports it.
}

fn check_terminator(
    term: &Terminator,
    param_counts: &HashMap<&str, usize>,
    errors: &mut Vec<VerifyError>,
) {
    match term {
        Terminator::TailCall(tc) => check_tail_call(tc, param_counts, errors),
        Terminator::Branch { then_call, else_call, .. } => {
            check_tail_call(then_call, param_counts, errors);
            check_tail_call(else_call, param_counts, errors);
        }
        Terminator::Return(_) | Terminator::Revert | Terminator::Unreachable => {}
    }
}
