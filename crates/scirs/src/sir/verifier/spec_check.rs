//! SIR verifier pass: spec_check
//!
//! Checks that `requires`/`ensures` clauses reference only params, `old()`, and `result`.

use crate::sir::utils::visit::{self, Visit};
use crate::sir::*;
use crate::verify::VerifyError;
use std::collections::HashSet;

const PASS: &str = "sir::spec_check";

struct SpecChecker {
    errors: Vec<VerifyError>,
    allowed_vars: HashSet<String>,
    in_spec: bool,
}

impl<'a> Visit<'a> for SpecChecker {
    fn visit_function_decl(&mut self, func: &'a FunctionDecl) {
        if let Some(spec) = &func.spec {
            self.allowed_vars.clear();
            for p in &func.params {
                self.allowed_vars.insert(p.name.clone());
            }
            self.in_spec = true;
            for r in &spec.requires {
                self.visit_expr(r);
            }
            for e in &spec.ensures {
                self.visit_expr(e);
            }
            if let Some(d) = &spec.decreases {
                self.visit_expr(d);
            }
            self.in_spec = false;
        }
        // Visit the body normally (not in spec context)
        if let Some(body) = &func.body {
            self.visit_stmts(body);
        }
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        if self.in_spec {
            if let Expr::Var(v) = expr {
                if !self.allowed_vars.contains(&v.name) {
                    let mut err = VerifyError::new(
                        PASS,
                        format!(
                            "spec clause references variable `{}` which is not a parameter",
                            v.name,
                        ),
                    );
                    if let Some(span) = v.span {
                        err = err.with_span(span);
                    }
                    self.errors.push(err);
                }
            }
        }
        visit::default::visit_expr(self, expr);
    }
}

pub fn check(module: &Module) -> Vec<VerifyError> {
    let mut checker =
        SpecChecker { errors: Vec::new(), allowed_vars: HashSet::new(), in_spec: false };
    checker.visit_module(module);
    checker.errors
}
