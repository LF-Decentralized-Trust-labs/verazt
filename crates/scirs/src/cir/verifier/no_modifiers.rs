//! CIR verifier pass: no_modifiers
//!
//! Verifies that no modifier definitions remain (inlined by `elim_modifiers`).
//! CIR `CanonMemberDecl` has no `Modifier` variant — structurally guaranteed.

use crate::cir::utils::visit::Visit;
use crate::cir::*;
use crate::verify::VerifyError;

const _PASS: &str = "cir::no_modifiers";

struct NoModifiersChecker {
    errors: Vec<VerifyError>,
}

impl<'a> Visit<'a> for NoModifiersChecker {
    fn visit_member_decl(&mut self, member: &'a CanonMemberDecl) {
        // CanonMemberDecl has no Modifier variant — structurally guaranteed.
        let _ = member;
    }
}

pub fn check(module: &CanonModule) -> Vec<VerifyError> {
    let mut checker = NoModifiersChecker { errors: Vec::new() };
    checker.visit_module(module);
    checker.errors
}
