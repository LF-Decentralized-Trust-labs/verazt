//! CIR verifier — checks CIR canonicalization invariants.

mod atomic_call_args;
mod no_inheritance;
mod no_modifiers;
mod no_named_args;
mod no_ternary;
mod no_using_for;

use crate::cir::CanonModule;
use crate::verify::VerifyError;

/// Run all CIR verification passes on a module.
pub fn verify(module: &CanonModule, verbose: bool) -> Result<(), Vec<VerifyError>> {
    let mut errors = Vec::new();

    run_pass("no_named_args", no_named_args::check(module), verbose, &mut errors);
    run_pass("no_using_for", no_using_for::check(module), verbose, &mut errors);
    run_pass("no_modifiers", no_modifiers::check(module), verbose, &mut errors);
    run_pass("atomic_call_args", atomic_call_args::check(module), verbose, &mut errors);
    run_pass("no_ternary", no_ternary::check(module), verbose, &mut errors);
    run_pass("no_inheritance", no_inheritance::check(module), verbose, &mut errors);

    if errors.is_empty() { Ok(()) } else { Err(errors) }
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
