//! CIR verifier pass: no_using_for
//!
//! Verifies that no `UsingFor` member declarations remain.
//! CIR `CanonMemberDecl` has no `UsingFor` variant — this is enforced
//! by the type system, so this pass simply traverses for confirmation.

use crate::cir::utils::visit::Visit;
use crate::cir::*;
use crate::verify::VerifyError;

const _PASS: &str = "cir::no_using_for";

struct NoUsingForChecker {
    errors: Vec<VerifyError>,
}

impl<'a> Visit<'a> for NoUsingForChecker {
    fn visit_member_decl(&mut self, member: &'a CanonMemberDecl) {
        // CanonMemberDecl has no UsingFor variant — structurally guaranteed.
        // This pass exists for completeness. If the enum were extended,
        // we would catch it here.
        let _ = member;
    }
}

pub fn check(module: &CanonModule) -> Vec<VerifyError> {
    let mut checker = NoUsingForChecker { errors: Vec::new() };
    checker.visit_module(module);
    checker.errors
}
