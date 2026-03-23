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

    run_pass("type_well_formed", type_well_formed::check(module), verbose, &mut errors);
    run_pass("scope_check", scope_check::check(module), verbose, &mut errors);
    run_pass("spec_check", spec_check::check(module), verbose, &mut errors);
    run_pass("no_orphan_dialect", no_orphan_dialect::check(module), verbose, &mut errors);

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
