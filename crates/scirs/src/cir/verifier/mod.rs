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
pub fn verify(module: &CanonModule) -> Result<(), Vec<VerifyError>> {
    let mut errors = Vec::new();

    errors.extend(no_named_args::check(module));
    errors.extend(no_using_for::check(module));
    errors.extend(no_modifiers::check(module));
    errors.extend(atomic_call_args::check(module));
    errors.extend(no_ternary::check(module));
    errors.extend(no_inheritance::check(module));

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}
