//! FIR verifier pass: terminal_completeness
//!
//! Every function body ends with exactly one FIR Terminator.
//! This is structurally guaranteed since `Function.term` is a required field,
//! but we additionally verify that the body contains no embedded terminators.

use crate::bir::ops::OpKind;
use crate::fir::ops::Module;
use crate::verify::VerifyError;

const PASS: &str = "fir::terminal_completeness";

pub fn check(module: &Module) -> Vec<VerifyError> {
    let mut errors = Vec::new();

    for func in &module.functions {
        // Check that no ops in the body are Return ops — returns should only
        // be expressed via the Terminator.
        for op in &func.body {
            if matches!(op.kind, OpKind::Return(_)) {
                let mut err = VerifyError::new(
                    PASS,
                    format!(
                        "function {} body contains a Return op {} — use Terminator::Return instead",
                        func.id, op.id
                    ),
                );
                if let Some(span) = op.span {
                    err = err.with_span(span);
                }
                errors.push(err);
            }
        }
    }

    errors
}
