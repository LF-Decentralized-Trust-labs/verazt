//! Solidity AST verifier — checks AST structural invariants.

mod type_well_formed;

use crate::solidity::ast::SourceUnit;
use scirs::verify::VerifyError;

/// Run all AST verification passes on source units.
pub fn verify(source_units: &[SourceUnit], verbose: bool) -> Result<(), Vec<VerifyError>> {
    let mut errors = Vec::new();

    run_pass(1, "type_well_formed", type_well_formed::check(source_units), verbose, &mut errors);

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
