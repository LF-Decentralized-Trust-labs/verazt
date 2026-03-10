//! SIR query helpers.
//!
//! Generic SIR tree-walking utilities that any crate working with SIR types
//! can use.  They match only core grammar nodes and `dialect_attrs`, so they
//! are **cross-chain by construction**: the same rule fires for Solidity,
//! Vyper, Move, and Anchor contracts without any per-dialect branch.

use crate::sir::{BinOpExpr, ContractDecl, DialectExpr, Expr, FunctionDecl, MemberDecl, Stmt};

// =========================================================================
// Function attribute queries
// =========================================================================

/// Check if a SIR `FunctionDecl` has public visibility.
pub fn is_public_function(func: &FunctionDecl) -> bool {
    func.attrs.iter().any(|a| {
        a.namespace == "sir"
            && a.key == crate::sir::attrs::sir_attrs::VISIBILITY
            && matches!(&a.value, crate::sir::AttrValue::String(s) if s == "public")
    })
}

/// Check if a SIR `FunctionDecl` has a reentrancy guard.
pub fn has_reentrancy_guard(func: &FunctionDecl) -> bool {
    func.attrs.iter().any(|a| {
        (a.namespace == "sir" && a.key == crate::sir::attrs::sir_attrs::REENTRANCY_GUARD)
            || (a.namespace == "evm" && a.key == crate::sir::attrs::evm_attrs::NONREENTRANT)
    })
}

// =========================================================================
// Storage helpers
// =========================================================================

/// Collect all storage variable names from a contract.
pub fn storage_names(contract: &ContractDecl) -> Vec<String> {
    contract
        .members
        .iter()
        .filter_map(|m| {
            if let MemberDecl::Storage(s) = m {
                Some(s.name.clone())
            } else {
                None
            }
        })
        .collect()
}

/// Recursively check if any statement writes to storage (assign to a storage
/// name).
pub fn has_storage_write(stmts: &[Stmt], storage_vars: &[String]) -> bool {
    for stmt in stmts {
        match stmt {
            Stmt::Assign(a) => {
                if expr_references_storage(&a.lhs, storage_vars) {
                    return true;
                }
            }
            Stmt::AugAssign(a) => {
                if expr_references_storage(&a.lhs, storage_vars) {
                    return true;
                }
            }
            Stmt::If(s) => {
                if has_storage_write(&s.then_body, storage_vars) {
                    return true;
                }
                if let Some(else_body) = &s.else_body {
                    if has_storage_write(else_body, storage_vars) {
                        return true;
                    }
                }
            }
            Stmt::While(s) => {
                if has_storage_write(&s.body, storage_vars) {
                    return true;
                }
            }
            Stmt::For(s) => {
                if has_storage_write(&s.body, storage_vars) {
                    return true;
                }
            }
            Stmt::Block(stmts) => {
                if has_storage_write(stmts, storage_vars) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// Check if an expression references a storage variable.
pub fn expr_references_storage(expr: &Expr, storage_vars: &[String]) -> bool {
    match expr {
        Expr::Var(v) => storage_vars.contains(&v.name),
        Expr::IndexAccess(ia) => expr_references_storage(&ia.base, storage_vars),
        Expr::FieldAccess(fa) => expr_references_storage(&fa.base, storage_vars),
        _ => false,
    }
}

/// Check if a function body contains an Assert before the first storage write.
pub fn has_assert_before_storage_write(stmts: &[Stmt], storage_vars: &[String]) -> bool {
    for stmt in stmts {
        match stmt {
            Stmt::Assert(_) => return true,
            Stmt::Assign(a) => {
                if expr_references_storage(&a.lhs, storage_vars) {
                    return false;
                }
            }
            Stmt::AugAssign(a) => {
                if expr_references_storage(&a.lhs, storage_vars) {
                    return false;
                }
            }
            _ => {}
        }
    }
    false
}

// =========================================================================
// Tree walkers
// =========================================================================

/// Recursively walk all `Expr::BinOp` nodes in a statement tree.
pub fn walk_binops(stmts: &[Stmt], visitor: &mut dyn FnMut(&BinOpExpr)) {
    for stmt in stmts {
        walk_binops_stmt(stmt, visitor);
    }
}

fn walk_binops_stmt(stmt: &Stmt, visitor: &mut dyn FnMut(&BinOpExpr)) {
    match stmt {
        Stmt::Assign(a) => {
            walk_binops_expr(&a.lhs, visitor);
            walk_binops_expr(&a.rhs, visitor);
        }
        Stmt::AugAssign(a) => {
            walk_binops_expr(&a.lhs, visitor);
            walk_binops_expr(&a.rhs, visitor);
        }
        Stmt::LocalVar(lv) => {
            if let Some(init) = &lv.init {
                walk_binops_expr(init, visitor);
            }
        }
        Stmt::Expr(e) => walk_binops_expr(&e.expr, visitor),
        Stmt::If(s) => {
            walk_binops_expr(&s.cond, visitor);
            walk_binops_stmts(&s.then_body, visitor);
            if let Some(else_body) = &s.else_body {
                walk_binops_stmts(else_body, visitor);
            }
        }
        Stmt::While(s) => {
            walk_binops_expr(&s.cond, visitor);
            walk_binops_stmts(&s.body, visitor);
        }
        Stmt::For(s) => {
            if let Some(init) = &s.init {
                walk_binops_stmt(init, visitor);
            }
            if let Some(cond) = &s.cond {
                walk_binops_expr(cond, visitor);
            }
            if let Some(update) = &s.update {
                walk_binops_stmt(update, visitor);
            }
            walk_binops_stmts(&s.body, visitor);
        }
        Stmt::Return(r) => {
            if let Some(v) = &r.value {
                walk_binops_expr(v, visitor);
            }
        }
        Stmt::Assert(a) => {
            walk_binops_expr(&a.cond, visitor);
        }
        Stmt::Block(stmts) => walk_binops_stmts(stmts, visitor),
        _ => {}
    }
}

fn walk_binops_stmts(stmts: &[Stmt], visitor: &mut dyn FnMut(&BinOpExpr)) {
    for stmt in stmts {
        walk_binops_stmt(stmt, visitor);
    }
}

fn walk_binops_expr(expr: &Expr, visitor: &mut dyn FnMut(&BinOpExpr)) {
    match expr {
        Expr::BinOp(binop) => {
            visitor(binop);
            walk_binops_expr(&binop.lhs, visitor);
            walk_binops_expr(&binop.rhs, visitor);
        }
        Expr::UnOp(u) => walk_binops_expr(&u.operand, visitor),
        Expr::IndexAccess(ia) => {
            walk_binops_expr(&ia.base, visitor);
            if let Some(idx) = &ia.index {
                walk_binops_expr(idx, visitor);
            }
        }
        Expr::FieldAccess(fa) => walk_binops_expr(&fa.base, visitor),
        Expr::FunctionCall(c) => {
            walk_binops_expr(&c.callee, visitor);
            for arg in &c.args {
                walk_binops_expr(arg, visitor);
            }
        }
        Expr::TypeCast(tc) => walk_binops_expr(&tc.expr, visitor),
        Expr::Ternary(t) => {
            walk_binops_expr(&t.cond, visitor);
            walk_binops_expr(&t.then_expr, visitor);
            walk_binops_expr(&t.else_expr, visitor);
        }
        Expr::Tuple(t) => {
            for elem in &t.elems {
                if let Some(e) = elem {
                    walk_binops_expr(e, visitor);
                }
            }
        }
        _ => {}
    }
}

/// Recursively walk all dialect expressions.
pub fn walk_dialect_exprs(stmts: &[Stmt], visitor: &mut dyn FnMut(&DialectExpr)) {
    for stmt in stmts {
        walk_dialect_exprs_stmt(stmt, visitor);
    }
}

fn walk_dialect_exprs_stmt(stmt: &Stmt, visitor: &mut dyn FnMut(&DialectExpr)) {
    match stmt {
        Stmt::Assign(a) => {
            walk_dialect_exprs_expr(&a.lhs, visitor);
            walk_dialect_exprs_expr(&a.rhs, visitor);
        }
        Stmt::AugAssign(a) => {
            walk_dialect_exprs_expr(&a.lhs, visitor);
            walk_dialect_exprs_expr(&a.rhs, visitor);
        }
        Stmt::LocalVar(lv) => {
            if let Some(init) = &lv.init {
                walk_dialect_exprs_expr(init, visitor);
            }
        }
        Stmt::Expr(e) => walk_dialect_exprs_expr(&e.expr, visitor),
        Stmt::If(s) => {
            walk_dialect_exprs_expr(&s.cond, visitor);
            for st in &s.then_body {
                walk_dialect_exprs_stmt(st, visitor);
            }
            if let Some(else_body) = &s.else_body {
                for st in else_body {
                    walk_dialect_exprs_stmt(st, visitor);
                }
            }
        }
        Stmt::While(s) => {
            walk_dialect_exprs_expr(&s.cond, visitor);
            for st in &s.body {
                walk_dialect_exprs_stmt(st, visitor);
            }
        }
        Stmt::For(s) => {
            if let Some(init) = &s.init {
                walk_dialect_exprs_stmt(init, visitor);
            }
            if let Some(cond) = &s.cond {
                walk_dialect_exprs_expr(cond, visitor);
            }
            if let Some(update) = &s.update {
                walk_dialect_exprs_stmt(update, visitor);
            }
            for st in &s.body {
                walk_dialect_exprs_stmt(st, visitor);
            }
        }
        Stmt::Return(r) => {
            if let Some(v) = &r.value {
                walk_dialect_exprs_expr(v, visitor);
            }
        }
        Stmt::Assert(a) => {
            walk_dialect_exprs_expr(&a.cond, visitor);
        }
        Stmt::Block(stmts) => {
            for st in stmts {
                walk_dialect_exprs_stmt(st, visitor);
            }
        }
        _ => {}
    }
}

fn walk_dialect_exprs_expr(expr: &Expr, visitor: &mut dyn FnMut(&DialectExpr)) {
    match expr {
        Expr::Dialect(d) => visitor(d),
        Expr::BinOp(b) => {
            walk_dialect_exprs_expr(&b.lhs, visitor);
            walk_dialect_exprs_expr(&b.rhs, visitor);
        }
        Expr::UnOp(u) => walk_dialect_exprs_expr(&u.operand, visitor),
        Expr::IndexAccess(ia) => {
            walk_dialect_exprs_expr(&ia.base, visitor);
            if let Some(idx) = &ia.index {
                walk_dialect_exprs_expr(idx, visitor);
            }
        }
        Expr::FieldAccess(fa) => walk_dialect_exprs_expr(&fa.base, visitor),
        Expr::FunctionCall(c) => {
            walk_dialect_exprs_expr(&c.callee, visitor);
            for arg in &c.args {
                walk_dialect_exprs_expr(arg, visitor);
            }
        }
        Expr::TypeCast(tc) => walk_dialect_exprs_expr(&tc.expr, visitor),
        Expr::Ternary(t) => {
            walk_dialect_exprs_expr(&t.cond, visitor);
            walk_dialect_exprs_expr(&t.then_expr, visitor);
            walk_dialect_exprs_expr(&t.else_expr, visitor);
        }
        Expr::Tuple(t) => {
            for elem in &t.elems {
                if let Some(e) = elem {
                    walk_dialect_exprs_expr(e, visitor);
                }
            }
        }
        _ => {}
    }
}
