//! FIR verifier: orchestrates all FIR verification passes.

use crate::fir::ops::Module;
use crate::verify::VerifyError;

pub mod no_orphan_blocks;
pub mod no_phi;
pub mod param_arity;
pub mod tail_call_targets;
pub mod terminal_completeness;

/// Run all FIR verification passes and return collected errors.
pub fn verify(module: &Module) -> Result<(), Vec<VerifyError>> {
    let mut errors = Vec::new();

    errors.extend(no_phi::check(module));
    errors.extend(tail_call_targets::check(module));
    errors.extend(param_arity::check(module));
    errors.extend(no_orphan_blocks::check(module));
    errors.extend(terminal_completeness::check(module));

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
