//! SIR query helpers.
//!
//! Generic SIR tree-walking utilities that any crate working with SIR types
//! can use.  They match only core grammar nodes and `dialect_attrs`, so they
//! are **cross-chain by construction**: the same rule fires for Solidity,
//! Vyper, Move, and Anchor contracts without any per-dialect branch.

use crate::sir::dialect::evm::EvmExpr;
use crate::sir::{
    BinOpExpr, CallExpr, ContractDecl, DialectExpr, Expr, FunctionDecl, MemberDecl, Stmt,
    StorageDecl,
};

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

// =========================================================================
// Generic expression and statement walkers (Phase 1.1)
// =========================================================================

/// Walk every `Expr` in a statement tree, bottom-up.
pub fn walk_exprs(stmts: &[Stmt], visitor: &mut dyn FnMut(&Expr)) {
    for stmt in stmts {
        walk_exprs_stmt(stmt, visitor);
    }
}

fn walk_exprs_stmt(stmt: &Stmt, visitor: &mut dyn FnMut(&Expr)) {
    match stmt {
        Stmt::Assign(a) => {
            walk_exprs_expr(&a.lhs, visitor);
            walk_exprs_expr(&a.rhs, visitor);
        }
        Stmt::AugAssign(a) => {
            walk_exprs_expr(&a.lhs, visitor);
            walk_exprs_expr(&a.rhs, visitor);
        }
        Stmt::LocalVar(lv) => {
            if let Some(init) = &lv.init {
                walk_exprs_expr(init, visitor);
            }
        }
        Stmt::Expr(e) => walk_exprs_expr(&e.expr, visitor),
        Stmt::If(s) => {
            walk_exprs_expr(&s.cond, visitor);
            walk_exprs(&&s.then_body, visitor);
            if let Some(else_body) = &s.else_body {
                walk_exprs(else_body, visitor);
            }
        }
        Stmt::While(s) => {
            walk_exprs_expr(&s.cond, visitor);
            walk_exprs(&s.body, visitor);
        }
        Stmt::For(s) => {
            if let Some(init) = &s.init {
                walk_exprs_stmt(init, visitor);
            }
            if let Some(cond) = &s.cond {
                walk_exprs_expr(cond, visitor);
            }
            if let Some(update) = &s.update {
                walk_exprs_stmt(update, visitor);
            }
            walk_exprs(&s.body, visitor);
        }
        Stmt::Return(r) => {
            if let Some(v) = &r.value {
                walk_exprs_expr(v, visitor);
            }
        }
        Stmt::Assert(a) => {
            walk_exprs_expr(&a.cond, visitor);
        }
        Stmt::Revert(r) => {
            for arg in &r.args {
                walk_exprs_expr(arg, visitor);
            }
        }
        Stmt::Block(stmts) => walk_exprs(stmts, visitor),
        Stmt::Dialect(_) | Stmt::Break | Stmt::Continue => {}
    }
}

fn walk_exprs_expr(expr: &Expr, visitor: &mut dyn FnMut(&Expr)) {
    // Recurse into sub-expressions first (bottom-up)
    match expr {
        Expr::BinOp(b) => {
            walk_exprs_expr(&b.lhs, visitor);
            walk_exprs_expr(&b.rhs, visitor);
        }
        Expr::UnOp(u) => walk_exprs_expr(&u.operand, visitor),
        Expr::IndexAccess(ia) => {
            walk_exprs_expr(&ia.base, visitor);
            if let Some(idx) = &ia.index {
                walk_exprs_expr(idx, visitor);
            }
        }
        Expr::FieldAccess(fa) => walk_exprs_expr(&fa.base, visitor),
        Expr::FunctionCall(c) => {
            walk_exprs_expr(&c.callee, visitor);
            for arg in &c.args {
                walk_exprs_expr(arg, visitor);
            }
        }
        Expr::TypeCast(tc) => walk_exprs_expr(&tc.expr, visitor),
        Expr::Ternary(t) => {
            walk_exprs_expr(&t.cond, visitor);
            walk_exprs_expr(&t.then_expr, visitor);
            walk_exprs_expr(&t.else_expr, visitor);
        }
        Expr::Tuple(t) => {
            for elem in &t.elems {
                if let Some(e) = elem {
                    walk_exprs_expr(e, visitor);
                }
            }
        }
        _ => {}
    }
    // Visit node after children
    visitor(expr);
}

/// Walk every `Stmt` in a tree, pre-order.
pub fn walk_stmts(stmts: &[Stmt], visitor: &mut dyn FnMut(&Stmt)) {
    for stmt in stmts {
        walk_stmts_one(stmt, visitor);
    }
}

fn walk_stmts_one(stmt: &Stmt, visitor: &mut dyn FnMut(&Stmt)) {
    visitor(stmt);
    match stmt {
        Stmt::If(s) => {
            walk_stmts(&s.then_body, visitor);
            if let Some(else_body) = &s.else_body {
                walk_stmts(else_body, visitor);
            }
        }
        Stmt::While(s) => walk_stmts(&s.body, visitor),
        Stmt::For(s) => {
            if let Some(init) = &s.init {
                walk_stmts_one(init, visitor);
            }
            if let Some(update) = &s.update {
                walk_stmts_one(update, visitor);
            }
            walk_stmts(&s.body, visitor);
        }
        Stmt::Block(stmts) => walk_stmts(stmts, visitor),
        _ => {}
    }
}

/// Walk every `Expr::FunctionCall` node.
pub fn walk_function_calls(stmts: &[Stmt], visitor: &mut dyn FnMut(&CallExpr)) {
    walk_exprs(stmts, &mut |expr| {
        if let Expr::FunctionCall(call) = expr {
            visitor(call);
        }
    });
}

/// True if the `CallExpr` represents an EVM external call:
/// `.call`, `.delegatecall`, `.staticcall`, `.transfer`, `.send`,
/// or an EVM dialect call variant.
pub fn is_evm_external_call(call: &CallExpr) -> bool {
    // Check for FieldAccess-based external calls (e.g. addr.call(...))
    if let Expr::FieldAccess(fa) = &*call.callee {
        let field = fa.field.as_str();
        if matches!(field, "call" | "delegatecall" | "staticcall" | "transfer" | "send") {
            return true;
        }
    }
    // Check for dialect expression-based calls
    if let Expr::Dialect(DialectExpr::Evm(evm)) = &*call.callee {
        match evm {
            EvmExpr::RawCall { .. }
            | EvmExpr::Send { .. }
            | EvmExpr::Delegatecall { .. }
            | EvmExpr::LowLevelCall { .. } => return true,
            _ => {}
        }
    }
    false
}

/// True if a `StorageDecl` carries a `#evm.is_constant` or `#evm.is_immutable`
/// attr.
pub fn is_constant_storage(storage: &StorageDecl) -> bool {
    storage.attrs.iter().any(|a| {
        a.namespace == "evm"
            && matches!(
                a.key.as_str(),
                crate::sir::attrs::evm_attrs::IS_CONSTANT
                    | crate::sir::attrs::evm_attrs::IS_IMMUTABLE
            )
            && matches!(&a.value, crate::sir::AttrValue::Bool(true))
    })
}
