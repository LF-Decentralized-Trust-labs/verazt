//! FIR verifier pass: no_phi
//!
//! No Phi ops remain in FIR (eliminated into function params).

use crate::bir::ops::OpKind;
use crate::fir::ops::Module;
use crate::verify::VerifyError;

const PASS: &str = "fir::no_phi";

pub fn check(module: &Module) -> Vec<VerifyError> {
    let mut errors = Vec::new();

    for func in &module.functions {
        for op in &func.body {
            if matches!(op.kind, OpKind::Phi(_)) {
                let mut err = VerifyError::new(
                    PASS,
                    format!("phi op {} found in FIR function {}", op.id, func.id),
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
