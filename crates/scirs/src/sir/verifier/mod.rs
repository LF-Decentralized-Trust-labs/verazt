//! SIR verifier — checks SIR structural invariants.

mod no_orphan_dialect;
mod scope_check;
mod spec_check;
mod type_well_formed;

use crate::sir::Module;
use crate::verify::VerifyError;

/// Run all SIR verification passes on a module.
pub fn verify(module: &Module) -> Result<(), Vec<VerifyError>> {
    let mut errors = Vec::new();

    errors.extend(type_well_formed::check(module));
    errors.extend(scope_check::check(module));
    errors.extend(spec_check::check(module));
    errors.extend(no_orphan_dialect::check(module));

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}
