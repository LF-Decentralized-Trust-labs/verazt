//! AST verifier pass: type_well_formed
//!
//! Checks that every expression and variable declaration has a resolved type
//! (i.e. no leftover `UserDefined` types that should have been resolved by the
//! type parser).

use crate::solidity::ast::utils::visit::{self, Visit};
use crate::solidity::ast::*;
use scirs::verify::VerifyError;

const PASS: &str = "ast::type_well_formed";

struct TypeWellFormedChecker {
    errors: Vec<VerifyError>,
}

impl TypeWellFormedChecker {
    fn check_type(&mut self, typ: &Type, context: &str) {
        if let Type::UserDefined(udt) = typ {
            self.errors.push(VerifyError::new(
                PASS,
                format!("{context} has unresolved UserDefined type `{udt}`"),
            ));
        }
    }
}

impl<'a> Visit<'a> for TypeWellFormedChecker {
    fn visit_expr(&mut self, expr: &'a Expr) {
        self.check_type(&expr.typ(), "expression");
        visit::default::visit_expr(self, expr);
    }

    fn visit_var_decl(&mut self, var: &'a VarDecl) {
        self.check_type(&var.typ, &format!("variable `{}`", var.name));
        visit::default::visit_var_decl(self, var);
    }
}

pub fn check(source_units: &[SourceUnit]) -> Vec<VerifyError> {
    let mut checker = TypeWellFormedChecker { errors: Vec::new() };
    checker.visit_source_units(source_units);
    checker.errors
}
