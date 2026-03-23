//! CIR verifier pass: no_inheritance
//!
//! Verifies that inheritance has been resolved — `CanonContractDecl`
//! has no `parents` field, so this is structurally guaranteed.

use crate::cir::*;
use crate::verify::VerifyError;

const _PASS: &str = "cir::no_inheritance";

/// Always returns an empty list — `CanonContractDecl` has no `parents` field.
pub fn check(_module: &CanonModule) -> Vec<VerifyError> {
    // Structurally guaranteed: CanonContractDecl has no `parents` field.
    Vec::new()
}
