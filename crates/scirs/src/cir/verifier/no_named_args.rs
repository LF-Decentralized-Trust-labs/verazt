//! CIR verifier pass: no_named_args
//!
//! Verifies that no named arguments remain — all call args are positional.
//! Since CIR `CanonCallExpr.args` is always `Vec<CanonExpr>`, this pass
//! confirms the structure is correct by traversing all call expressions.

use crate::cir::utils::visit::{self, Visit};
use crate::cir::*;
use crate::verify::VerifyError;

const _PASS: &str = "cir::no_named_args";

struct NoNamedArgsChecker {
    errors: Vec<VerifyError>,
}

impl<'a> Visit<'a> for NoNamedArgsChecker {
    fn visit_call_expr(&mut self, expr: &'a CanonCallExpr) {
        // CanonCallExpr.args is Vec<CanonExpr> (already positional).
        // This pass exists for structural completeness — the type system
        // already enforces this invariant in CIR.
        visit::default::visit_call_expr(self, expr);
    }
}

pub fn check(module: &CanonModule) -> Vec<VerifyError> {
    let mut checker = NoNamedArgsChecker { errors: Vec::new() };
    checker.visit_module(module);
    checker.errors
}
