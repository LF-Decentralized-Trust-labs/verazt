//! SIR verifier pass: no_orphan_dialect
//!
//! Checks that no `Dialect` variants appear outside of recognized dialect contexts.

use crate::sir::utils::visit::Visit;
use crate::sir::*;
use crate::verify::VerifyError;

const PASS: &str = "sir::no_orphan_dialect";

struct NoOrphanDialectChecker {
    errors: Vec<VerifyError>,
}

impl<'a> Visit<'a> for NoOrphanDialectChecker {
    fn visit_dialect_expr(&mut self, _expr: &'a DialectExpr) {
        self.errors.push(VerifyError::new(
            PASS,
            "orphan DialectExpr found outside recognized dialect context",
        ));
    }

    fn visit_dialect_stmt(&mut self, _stmt: &'a DialectStmt) {
        self.errors.push(VerifyError::new(
            PASS,
            "orphan DialectStmt found outside recognized dialect context",
        ));
    }

    fn visit_dialect_member_decl(&mut self, _decl: &'a DialectMemberDecl) {
        // Dialect member decls are expected in contracts; not orphan by default.
    }
}

pub fn check(module: &Module) -> Vec<VerifyError> {
    let mut checker = NoOrphanDialectChecker { errors: Vec::new() };
    checker.visit_module(module);
    checker.errors
}
