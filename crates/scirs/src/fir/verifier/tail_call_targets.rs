//! FIR verifier pass: tail_call_targets
//!
//! Every TailCall callee references a function in the same module.

use crate::fir::ops::{Module, TailCallData, Terminator};
use crate::verify::VerifyError;
use std::collections::HashSet;

const PASS: &str = "fir::tail_call_targets";

pub fn check(module: &Module) -> Vec<VerifyError> {
    let mut errors = Vec::new();

    let func_ids: HashSet<&str> = module.functions.iter().map(|f| f.id.0.as_str()).collect();

    for func in &module.functions {
        check_terminator(&func.term, &func_ids, &mut errors);
    }

    errors
}

fn check_tail_call(tc: &TailCallData, func_ids: &HashSet<&str>, errors: &mut Vec<VerifyError>) {
    if !func_ids.contains(tc.callee.0.as_str()) {
        errors.push(VerifyError::new(
            PASS,
            format!("tail call target {} is not a function in the module", tc.callee),
        ));
    }
}

fn check_terminator(term: &Terminator, func_ids: &HashSet<&str>, errors: &mut Vec<VerifyError>) {
    match term {
        Terminator::TailCall(tc) => check_tail_call(tc, func_ids, errors),
        Terminator::Branch { then_call, else_call, .. } => {
            check_tail_call(then_call, func_ids, errors);
            check_tail_call(else_call, func_ids, errors);
        }
        Terminator::Return(_) | Terminator::Revert | Terminator::Unreachable => {}
    }
}
