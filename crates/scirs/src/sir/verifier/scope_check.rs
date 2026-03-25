//! SIR verifier pass: scope_check
//!
//! Checks that variables used in expressions are declared in an enclosing
//! scope.

use crate::sir::*;
use crate::verify::VerifyError;
use std::collections::HashSet;

const PASS: &str = "sir::scope_check";

struct ScopeChecker {
    scopes: Vec<HashSet<String>>,
    errors: Vec<VerifyError>,
}

impl ScopeChecker {
    fn new() -> Self {
        ScopeChecker { scopes: vec![HashSet::new()], errors: Vec::new() }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashSet::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string());
        }
    }

    fn is_declared(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|s| s.contains(name))
    }

    fn check_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Var(v) => {
                if !self.is_declared(&v.name) {
                    let mut err = VerifyError::new(
                        PASS,
                        format!("variable `{}` used but not declared in scope", v.name),
                    );
                    if let Some(span) = v.span {
                        err = err.with_span(span);
                    }
                    self.errors.push(err);
                }
            }
            Expr::BinOp(e) => {
                self.check_expr(&e.lhs);
                self.check_expr(&e.rhs);
            }
            Expr::UnOp(e) => self.check_expr(&e.operand),
            Expr::IndexAccess(e) => {
                self.check_expr(&e.base);
                if let Some(idx) = &e.index {
                    self.check_expr(idx);
                }
            }
            Expr::FieldAccess(e) => self.check_expr(&e.base),
            Expr::FunctionCall(e) => {
                self.check_expr(&e.callee);
                for arg in e.args.exprs() {
                    self.check_expr(arg);
                }
            }
            Expr::TypeCast(e) => self.check_expr(&e.expr),
            Expr::Ternary(e) => {
                self.check_expr(&e.cond);
                self.check_expr(&e.then_expr);
                self.check_expr(&e.else_expr);
            }
            Expr::Tuple(e) => {
                for elem in &e.elems {
                    if let Some(ex) = elem {
                        self.check_expr(ex);
                    }
                }
            }
            Expr::Old(inner) => self.check_expr(inner),
            Expr::Forall { var, body, .. } | Expr::Exists { var, body, .. } => {
                self.push_scope();
                self.declare(var);
                self.check_expr(body);
                self.pop_scope();
            }
            Expr::Lit(_) | Expr::Result(_) | Expr::Dialect(_) => {}
        }
    }

    fn check_stmts(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            self.check_stmt(stmt);
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::LocalVar(s) => {
                if let Some(init) = &s.init {
                    self.check_expr(init);
                }
                for v in &s.vars {
                    if let Some(decl) = v {
                        self.declare(&decl.name);
                    }
                }
            }
            Stmt::Assign(s) => {
                self.check_expr(&s.lhs);
                self.check_expr(&s.rhs);
            }
            Stmt::AugAssign(s) => {
                self.check_expr(&s.lhs);
                self.check_expr(&s.rhs);
            }
            Stmt::Expr(s) => self.check_expr(&s.expr),
            Stmt::If(s) => {
                self.check_expr(&s.cond);
                self.push_scope();
                self.check_stmts(&s.then_body);
                self.pop_scope();
                if let Some(else_body) = &s.else_body {
                    self.push_scope();
                    self.check_stmts(else_body);
                    self.pop_scope();
                }
            }
            Stmt::While(s) => {
                self.check_expr(&s.cond);
                self.push_scope();
                self.check_stmts(&s.body);
                self.pop_scope();
            }
            Stmt::For(s) => {
                self.push_scope();
                if let Some(init) = &s.init {
                    self.check_stmt(init);
                }
                if let Some(cond) = &s.cond {
                    self.check_expr(cond);
                }
                if let Some(update) = &s.update {
                    self.check_stmt(update);
                }
                self.check_stmts(&s.body);
                self.pop_scope();
            }
            Stmt::Return(s) => {
                if let Some(v) = &s.value {
                    self.check_expr(v);
                }
            }
            Stmt::Revert(s) => {
                for arg in &s.args {
                    self.check_expr(arg);
                }
            }
            Stmt::Assert(s) => {
                self.check_expr(&s.cond);
                if let Some(msg) = &s.message {
                    self.check_expr(msg);
                }
            }
            Stmt::Block(stmts) => {
                self.push_scope();
                self.check_stmts(stmts);
                self.pop_scope();
            }
            Stmt::Break | Stmt::Continue | Stmt::Dialect(_) => {}
        }
    }

    fn check_function(&mut self, func: &FunctionDecl) {
        self.push_scope();
        for p in &func.params {
            self.declare(&p.name);
        }
        if let Some(body) = &func.body {
            self.check_stmts(body);
        }
        self.pop_scope();
    }

    fn check_module(&mut self, module: &Module) {
        for decl in &module.decls {
            if let Decl::Contract(contract) = decl {
                self.push_scope();
                // Declare storage variables as in-scope
                for m in &contract.members {
                    if let MemberDecl::Storage(s) = m {
                        self.declare(&s.name);
                    }
                }
                for m in &contract.members {
                    if let MemberDecl::Function(f) = m {
                        self.check_function(f);
                    }
                }
                self.pop_scope();
            }
        }
    }
}

pub fn check(module: &Module) -> Vec<VerifyError> {
    let mut checker = ScopeChecker::new();
    checker.check_module(module);
    checker.errors
}
