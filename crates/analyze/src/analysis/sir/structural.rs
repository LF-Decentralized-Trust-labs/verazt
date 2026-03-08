//! Shared trait and helpers for SIR structural passes.
//!
//! SIR structural passes pattern-match over `mlir::sir::Module` trees
//! **before** SSA or dialect lowering. They use `PassRepresentation::Ir`,
//! depend only on `PassId::IrGeneration`, and run in the existing analysis
//! phase — no new pipeline phase is required.
//!
//! Because they match only core grammar nodes and `dialect_attrs`, they are
//! **cross-chain by construction**: the same rule fires for Solidity, Vyper,
//! Move, and Anchor contracts without any per-dialect branch.

/// Helper function to check if a SIR `FunctionDecl` has public visibility.
pub fn is_public_function(func: &mlir::sir::FunctionDecl) -> bool {
    func.attrs.iter().any(|a| {
        a.namespace == "sir"
            && a.key == mlir::sir::attrs::sir_attrs::VISIBILITY
            && matches!(&a.value, mlir::sir::AttrValue::String(s) if s == "public")
    })
}

/// Helper function to check if a SIR `FunctionDecl` has a reentrancy guard.
pub fn has_reentrancy_guard(func: &mlir::sir::FunctionDecl) -> bool {
    func.attrs.iter().any(|a| {
        (a.namespace == "sir" && a.key == mlir::sir::attrs::sir_attrs::REENTRANCY_GUARD)
            || (a.namespace == "evm" && a.key == mlir::sir::attrs::evm_attrs::NONREENTRANT)
    })
}

/// Helper: collect all storage variable names from a contract.
pub fn storage_names(contract: &mlir::sir::ContractDecl) -> Vec<String> {
    contract
        .members
        .iter()
        .filter_map(|m| {
            if let mlir::sir::MemberDecl::Storage(s) = m {
                Some(s.name.clone())
            } else {
                None
            }
        })
        .collect()
}

/// Recursively check if any statement writes to storage (assign to a storage
/// name).
pub fn has_storage_write(stmts: &[mlir::sir::Stmt], storage_vars: &[String]) -> bool {
    for stmt in stmts {
        match stmt {
            mlir::sir::Stmt::Assign(a) => {
                if expr_references_storage(&a.lhs, storage_vars) {
                    return true;
                }
            }
            mlir::sir::Stmt::AugAssign(a) => {
                if expr_references_storage(&a.lhs, storage_vars) {
                    return true;
                }
            }
            mlir::sir::Stmt::If(s) => {
                if has_storage_write(&s.then_body, storage_vars) {
                    return true;
                }
                if let Some(else_body) = &s.else_body {
                    if has_storage_write(else_body, storage_vars) {
                        return true;
                    }
                }
            }
            mlir::sir::Stmt::While(s) => {
                if has_storage_write(&s.body, storage_vars) {
                    return true;
                }
            }
            mlir::sir::Stmt::For(s) => {
                if has_storage_write(&s.body, storage_vars) {
                    return true;
                }
            }
            mlir::sir::Stmt::Block(stmts) => {
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
pub fn expr_references_storage(expr: &mlir::sir::Expr, storage_vars: &[String]) -> bool {
    match expr {
        mlir::sir::Expr::Var(v) => storage_vars.contains(&v.name),
        mlir::sir::Expr::IndexAccess(ia) => expr_references_storage(&ia.base, storage_vars),
        mlir::sir::Expr::FieldAccess(fa) => expr_references_storage(&fa.base, storage_vars),
        _ => false,
    }
}

/// Check if a function body contains an Assert before the first storage write.
pub fn has_assert_before_storage_write(stmts: &[mlir::sir::Stmt], storage_vars: &[String]) -> bool {
    for stmt in stmts {
        match stmt {
            mlir::sir::Stmt::Assert(_) => return true,
            mlir::sir::Stmt::Assign(a) => {
                if expr_references_storage(&a.lhs, storage_vars) {
                    return false;
                }
            }
            mlir::sir::Stmt::AugAssign(a) => {
                if expr_references_storage(&a.lhs, storage_vars) {
                    return false;
                }
            }
            _ => {}
        }
    }
    false
}

/// Recursively walk all `Expr::BinOp` nodes in a statement tree.
pub fn walk_binops(stmts: &[mlir::sir::Stmt], visitor: &mut dyn FnMut(&mlir::sir::BinOpExpr)) {
    for stmt in stmts {
        walk_binops_stmt(stmt, visitor);
    }
}

fn walk_binops_stmt(stmt: &mlir::sir::Stmt, visitor: &mut dyn FnMut(&mlir::sir::BinOpExpr)) {
    match stmt {
        mlir::sir::Stmt::Assign(a) => {
            walk_binops_expr(&a.lhs, visitor);
            walk_binops_expr(&a.rhs, visitor);
        }
        mlir::sir::Stmt::AugAssign(a) => {
            walk_binops_expr(&a.lhs, visitor);
            walk_binops_expr(&a.rhs, visitor);
        }
        mlir::sir::Stmt::LocalVar(lv) => {
            if let Some(init) = &lv.init {
                walk_binops_expr(init, visitor);
            }
        }
        mlir::sir::Stmt::Expr(e) => walk_binops_expr(&e.expr, visitor),
        mlir::sir::Stmt::If(s) => {
            walk_binops_expr(&s.cond, visitor);
            walk_binops_stmts(&s.then_body, visitor);
            if let Some(else_body) = &s.else_body {
                walk_binops_stmts(else_body, visitor);
            }
        }
        mlir::sir::Stmt::While(s) => {
            walk_binops_expr(&s.cond, visitor);
            walk_binops_stmts(&s.body, visitor);
        }
        mlir::sir::Stmt::For(s) => {
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
        mlir::sir::Stmt::Return(r) => {
            if let Some(v) = &r.value {
                walk_binops_expr(v, visitor);
            }
        }
        mlir::sir::Stmt::Assert(a) => {
            walk_binops_expr(&a.cond, visitor);
        }
        mlir::sir::Stmt::Block(stmts) => walk_binops_stmts(stmts, visitor),
        _ => {}
    }
}

fn walk_binops_stmts(stmts: &[mlir::sir::Stmt], visitor: &mut dyn FnMut(&mlir::sir::BinOpExpr)) {
    for stmt in stmts {
        walk_binops_stmt(stmt, visitor);
    }
}

fn walk_binops_expr(expr: &mlir::sir::Expr, visitor: &mut dyn FnMut(&mlir::sir::BinOpExpr)) {
    match expr {
        mlir::sir::Expr::BinOp(binop) => {
            visitor(binop);
            walk_binops_expr(&binop.lhs, visitor);
            walk_binops_expr(&binop.rhs, visitor);
        }
        mlir::sir::Expr::UnOp(u) => walk_binops_expr(&u.operand, visitor),
        mlir::sir::Expr::IndexAccess(ia) => {
            walk_binops_expr(&ia.base, visitor);
            if let Some(idx) = &ia.index {
                walk_binops_expr(idx, visitor);
            }
        }
        mlir::sir::Expr::FieldAccess(fa) => walk_binops_expr(&fa.base, visitor),
        mlir::sir::Expr::FunctionCall(c) => {
            walk_binops_expr(&c.callee, visitor);
            for arg in &c.args {
                walk_binops_expr(arg, visitor);
            }
        }
        mlir::sir::Expr::TypeCast(tc) => walk_binops_expr(&tc.expr, visitor),
        mlir::sir::Expr::Ternary(t) => {
            walk_binops_expr(&t.cond, visitor);
            walk_binops_expr(&t.then_expr, visitor);
            walk_binops_expr(&t.else_expr, visitor);
        }
        mlir::sir::Expr::Tuple(t) => {
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
pub fn walk_dialect_exprs(stmts: &[mlir::sir::Stmt], visitor: &mut dyn FnMut(&mlir::sir::DialectExpr)) {
    for stmt in stmts {
        walk_dialect_exprs_stmt(stmt, visitor);
    }
}

fn walk_dialect_exprs_stmt(stmt: &mlir::sir::Stmt, visitor: &mut dyn FnMut(&mlir::sir::DialectExpr)) {
    match stmt {
        mlir::sir::Stmt::Assign(a) => {
            walk_dialect_exprs_expr(&a.lhs, visitor);
            walk_dialect_exprs_expr(&a.rhs, visitor);
        }
        mlir::sir::Stmt::AugAssign(a) => {
            walk_dialect_exprs_expr(&a.lhs, visitor);
            walk_dialect_exprs_expr(&a.rhs, visitor);
        }
        mlir::sir::Stmt::LocalVar(lv) => {
            if let Some(init) = &lv.init {
                walk_dialect_exprs_expr(init, visitor);
            }
        }
        mlir::sir::Stmt::Expr(e) => walk_dialect_exprs_expr(&e.expr, visitor),
        mlir::sir::Stmt::If(s) => {
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
        mlir::sir::Stmt::While(s) => {
            walk_dialect_exprs_expr(&s.cond, visitor);
            for st in &s.body {
                walk_dialect_exprs_stmt(st, visitor);
            }
        }
        mlir::sir::Stmt::For(s) => {
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
        mlir::sir::Stmt::Return(r) => {
            if let Some(v) = &r.value {
                walk_dialect_exprs_expr(v, visitor);
            }
        }
        mlir::sir::Stmt::Assert(a) => {
            walk_dialect_exprs_expr(&a.cond, visitor);
        }
        mlir::sir::Stmt::Block(stmts) => {
            for st in stmts {
                walk_dialect_exprs_stmt(st, visitor);
            }
        }
        _ => {}
    }
}

fn walk_dialect_exprs_expr(expr: &mlir::sir::Expr, visitor: &mut dyn FnMut(&mlir::sir::DialectExpr)) {
    match expr {
        mlir::sir::Expr::Dialect(d) => visitor(d),
        mlir::sir::Expr::BinOp(b) => {
            walk_dialect_exprs_expr(&b.lhs, visitor);
            walk_dialect_exprs_expr(&b.rhs, visitor);
        }
        mlir::sir::Expr::UnOp(u) => walk_dialect_exprs_expr(&u.operand, visitor),
        mlir::sir::Expr::IndexAccess(ia) => {
            walk_dialect_exprs_expr(&ia.base, visitor);
            if let Some(idx) = &ia.index {
                walk_dialect_exprs_expr(idx, visitor);
            }
        }
        mlir::sir::Expr::FieldAccess(fa) => walk_dialect_exprs_expr(&fa.base, visitor),
        mlir::sir::Expr::FunctionCall(c) => {
            walk_dialect_exprs_expr(&c.callee, visitor);
            for arg in &c.args {
                walk_dialect_exprs_expr(arg, visitor);
            }
        }
        mlir::sir::Expr::TypeCast(tc) => walk_dialect_exprs_expr(&tc.expr, visitor),
        mlir::sir::Expr::Ternary(t) => {
            walk_dialect_exprs_expr(&t.cond, visitor);
            walk_dialect_exprs_expr(&t.then_expr, visitor);
            walk_dialect_exprs_expr(&t.else_expr, visitor);
        }
        mlir::sir::Expr::Tuple(t) => {
            for elem in &t.elems {
                if let Some(e) = elem {
                    walk_dialect_exprs_expr(e, visitor);
                }
            }
        }
        _ => {}
    }
}
