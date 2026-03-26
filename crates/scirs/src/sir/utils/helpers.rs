//! SIR domain helpers.
//!
//! Smart-contract domain predicates that any crate working with SIR types
//! can use.  They match only core grammar nodes and `dialect_attrs`, so they
//! are **cross-chain by construction**: the same rule fires for Solidity,
//! Vyper, Move, and Anchor contracts without any per-dialect branch.
//!
//! For generic tree traversal, use the `Visit<'a>` trait in
//! `scirs::sir::utils::visit` instead.

use crate::sir::dialect::evm::EvmExpr;
use crate::sir::{
    CallExpr, ContractDecl, DialectExpr, Expr, FunctionDecl, MemberDecl, Stmt, StorageDecl,
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
// EVM helpers
// =========================================================================

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
