//! CIR verifier pass: atomic_call_args
//!
//! Verifies that all function call arguments are atoms (Var or Lit),
//! not complex expressions.

use crate::cir::utils::visit::{self, Visit};
use crate::cir::*;
use crate::verify::VerifyError;

const PASS: &str = "cir::atomic_call_args";

struct AtomicCallArgsChecker {
    errors: Vec<VerifyError>,
}

fn is_atom(expr: &CanonExpr) -> bool {
    matches!(expr, CanonExpr::Var(_) | CanonExpr::Lit(_))
}

impl<'a> Visit<'a> for AtomicCallArgsChecker {
    fn visit_call_expr(&mut self, expr: &'a CanonCallExpr) {
        for (i, arg) in expr.args.iter().enumerate() {
            if !is_atom(arg) {
                let mut err = VerifyError::new(
                    PASS,
                    format!("call argument {i} is not an atom (Var or Lit)"),
                );
                if let Some(span) = expr.span {
                    err = err.with_span(span);
                }
                self.errors.push(err);
            }
        }
        visit::default::visit_call_expr(self, expr);
    }
}

pub fn check(module: &CanonModule) -> Vec<VerifyError> {
    let mut checker = AtomicCallArgsChecker { errors: Vec::new() };
    checker.visit_module(module);
    checker.errors
}
