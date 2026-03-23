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
pub fn verify(module: &Module) -> Result<(), Vec<VerifyError>> {
    let mut errors = Vec::new();

    errors.extend(ssa_single_def::check(module));
    errors.extend(ssa_use_def::check(module));
    errors.extend(cfg_well_formed::check(module));
    errors.extend(cfg_entry_exit::check(module));
    errors.extend(phi_consistency::check(module));
    errors.extend(op_id_unique::check(module));

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
