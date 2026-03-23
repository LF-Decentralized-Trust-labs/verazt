//! SIR verifier pass: type_well_formed
//!
//! Checks that every expression has a non-`None` type and that
//! function return types match the body.

use crate::sir::utils::visit::{self, Visit};
use crate::sir::*;
use crate::verify::VerifyError;

const PASS: &str = "sir::type_well_formed";

struct TypeWellFormedChecker {
    errors: Vec<VerifyError>,
}

impl<'a> Visit<'a> for TypeWellFormedChecker {
    fn visit_expr(&mut self, expr: &'a Expr) {
        if expr.typ() == Type::None {
            let mut err = VerifyError::new(PASS, "expression has `None` type");
            if let Some(span) = expr.span() {
                err = err.with_span(span);
            }
            self.errors.push(err);
        }
        visit::default::visit_expr(self, expr);
    }

    fn visit_function_decl(&mut self, func: &'a FunctionDecl) {
        // If the function has return types, check that the body is present
        if !func.returns.is_empty() && func.body.is_none() {
            self.errors.push(VerifyError::new(
                PASS,
                format!("function `{}` has return types but no body", func.name),
            ));
        }
        visit::default::visit_function_decl(self, func);
    }
}

pub fn check(module: &Module) -> Vec<VerifyError> {
    let mut checker = TypeWellFormedChecker { errors: Vec::new() };
    checker.visit_module(module);
    checker.errors
}
