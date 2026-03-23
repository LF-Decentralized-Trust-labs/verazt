//! FIR verifier: orchestrates all FIR verification passes.

use crate::fir::ops::Module;
use crate::verify::VerifyError;

pub mod no_orphan_blocks;
pub mod no_phi;
pub mod param_arity;
pub mod tail_call_targets;
pub mod terminal_completeness;

/// Run all FIR verification passes and return collected errors.
pub fn verify(module: &Module, verbose: bool) -> Result<(), Vec<VerifyError>> {
    let mut errors = Vec::new();

    run_pass("no_phi", no_phi::check(module), verbose, &mut errors);
    run_pass("tail_call_targets", tail_call_targets::check(module), verbose, &mut errors);
    run_pass("param_arity", param_arity::check(module), verbose, &mut errors);
    run_pass("no_orphan_blocks", no_orphan_blocks::check(module), verbose, &mut errors);
    run_pass("terminal_completeness", terminal_completeness::check(module), verbose, &mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn run_pass(name: &str, result: Vec<VerifyError>, verbose: bool, errors: &mut Vec<VerifyError>) {
    if verbose {
        if result.is_empty() {
            println!("- {name}: ✓");
        } else {
            println!("- {name}: ✗ ({} error(s))", result.len());
        }
    }
    errors.extend(result);
}
