//! CIR verifier pass: no_ternary
//!
//! Verifies that no ternary (conditional) expressions remain in CIR.
//! This is structurally guaranteed by the type system — `CanonExpr`
//! has no `Ternary` variant — so this pass always succeeds.

use crate::cir::*;
use crate::verify::VerifyError;

const _PASS: &str = "cir::no_ternary";

/// Always returns an empty list — CIR has no `Ternary` variant.
pub fn check(_module: &CanonModule) -> Vec<VerifyError> {
    // Structurally guaranteed: CanonExpr has no Ternary variant.
    Vec::new()
}
