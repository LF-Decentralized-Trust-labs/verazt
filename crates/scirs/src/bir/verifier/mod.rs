//! BIR verifier: orchestrates all BIR verification passes.

use crate::bir::module::Module;
use crate::verify::VerifyError;

pub mod cfg_entry_exit;
pub mod cfg_well_formed;
pub mod op_id_unique;
pub mod phi_consistency;
pub mod ssa_single_def;
pub mod ssa_use_def;

/// Run all BIR verification passes and return collected errors.
pub fn verify(module: &Module, verbose: bool) -> Result<(), Vec<VerifyError>> {
    let mut errors = Vec::new();

    run_pass(1, "ssa_single_def", ssa_single_def::check(module), verbose, &mut errors);
    run_pass(2, "ssa_use_def", ssa_use_def::check(module), verbose, &mut errors);
    run_pass(3, "cfg_well_formed", cfg_well_formed::check(module), verbose, &mut errors);
    run_pass(4, "cfg_entry_exit", cfg_entry_exit::check(module), verbose, &mut errors);
    run_pass(5, "phi_consistency", phi_consistency::check(module), verbose, &mut errors);
    run_pass(6, "op_id_unique", op_id_unique::check(module), verbose, &mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
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
