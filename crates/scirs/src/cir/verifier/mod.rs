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

    run_pass(1, "no_named_args", no_named_args::check(module), verbose, &mut errors);
    run_pass(2, "no_using_for", no_using_for::check(module), verbose, &mut errors);
    run_pass(3, "no_modifiers", no_modifiers::check(module), verbose, &mut errors);
    run_pass(4, "atomic_call_args", atomic_call_args::check(module), verbose, &mut errors);
    run_pass(5, "no_ternary", no_ternary::check(module), verbose, &mut errors);
    run_pass(6, "no_inheritance", no_inheritance::check(module), verbose, &mut errors);

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

fn run_pass(idx: usize, name: &str, result: Vec<VerifyError>, verbose: bool, errors: &mut Vec<VerifyError>) {
    if verbose {
        if result.is_empty() {
            println!("{idx}. {name}: ✓");
        } else {
            println!("{idx}. {name}: ✗ ({} errors)", result.len());
        }
    }
    errors.extend(result);
}
