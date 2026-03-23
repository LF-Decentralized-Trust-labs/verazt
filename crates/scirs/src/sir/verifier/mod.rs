//! SIR verifier — checks SIR structural invariants.

mod no_orphan_dialect;
mod scope_check;
mod spec_check;
mod type_well_formed;

use crate::sir::Module;
use crate::verify::VerifyError;

/// Run all SIR verification passes on a module.
pub fn verify(module: &Module, verbose: bool) -> Result<(), Vec<VerifyError>> {
    let mut errors = Vec::new();

    run_pass(1, "type_well_formed", type_well_formed::check(module), verbose, &mut errors);
    run_pass(2, "scope_check", scope_check::check(module), verbose, &mut errors);
    run_pass(3, "spec_check", spec_check::check(module), verbose, &mut errors);
    run_pass(4, "no_orphan_dialect", no_orphan_dialect::check(module), verbose, &mut errors);

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
