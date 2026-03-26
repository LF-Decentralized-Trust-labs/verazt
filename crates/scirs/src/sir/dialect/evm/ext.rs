//! EVM dialect extension traits for querying SIR node attributes.
//!
//! These traits provide ergonomic, method-style access to EVM-dialect-specific
//! metadata encoded as `Attr` annotations on SIR nodes.  Import the trait you
//! need and call methods directly on `FunctionDecl`, `CallExpr`, etc.

use crate::sir::attrs::{AttrValue, evm_attrs, sir_attrs};
use crate::sir::defs::{FunctionDecl, StorageDecl};
use crate::sir::dialect::DialectExpr;
use crate::sir::dialect::evm::EvmExpr;
use crate::sir::exprs::{CallExpr, Expr};

// ═══════════════════════════════════════════════════════════════════
// EvmFunctionExt — queries on FunctionDecl
// ═══════════════════════════════════════════════════════════════════

/// Extension trait for EVM-dialect queries on [`FunctionDecl`].
pub trait EvmFunctionExt {
    /// Returns `true` if the function has public visibility
    /// (`#sir.visibility = "public"`).
    fn is_public(&self) -> bool;

    /// Returns `true` if the function has a reentrancy guard
    /// (`#sir.reentrancy_guard` or `#evm.nonreentrant`).
    fn has_reentrancy_guard(&self) -> bool;
}

impl EvmFunctionExt for FunctionDecl {
    fn is_public(&self) -> bool {
        self.attrs.iter().any(|a| {
            a.namespace == "sir"
                && a.key == sir_attrs::VISIBILITY
                && matches!(&a.value, AttrValue::String(s) if s == "public")
        })
    }

    fn has_reentrancy_guard(&self) -> bool {
        self.attrs.iter().any(|a| {
            (a.namespace == "sir" && a.key == sir_attrs::REENTRANCY_GUARD)
                || (a.namespace == "evm" && a.key == evm_attrs::NONREENTRANT)
        })
    }
}

// ═══════════════════════════════════════════════════════════════════
// EvmCallExt — queries on CallExpr
// ═══════════════════════════════════════════════════════════════════

/// Extension trait for EVM-dialect queries on [`CallExpr`].
pub trait EvmCallExt {
    /// Returns `true` if the call is an EVM external call:
    /// `.call`, `.delegatecall`, `.staticcall`, `.transfer`, `.send`,
    /// or an EVM dialect call variant.
    fn is_evm_external_call(&self) -> bool;
}

impl EvmCallExt for CallExpr {
    fn is_evm_external_call(&self) -> bool {
        // Check for FieldAccess-based external calls (e.g. addr.call(...))
        if let Expr::FieldAccess(fa) = &*self.callee {
            let field = fa.field.as_str();
            if matches!(field, "call" | "delegatecall" | "staticcall" | "transfer" | "send") {
                return true;
            }
        }
        // Check for dialect expression-based calls
        if let Expr::Dialect(DialectExpr::Evm(evm)) = &*self.callee {
            match evm {
                EvmExpr::RawCall(_)
                | EvmExpr::Send(_)
                | EvmExpr::Delegatecall(_)
                | EvmExpr::LowLevelCall(_) => return true,
                _ => {}
            }
        }
        false
    }
}

// ═══════════════════════════════════════════════════════════════════
// EvmStorageExt — queries on StorageDecl
// ═══════════════════════════════════════════════════════════════════

/// Extension trait for EVM-dialect queries on [`StorageDecl`].
pub trait EvmStorageExt {
    /// Returns `true` if the storage variable carries a `#evm.is_constant`
    /// or `#evm.is_immutable` attribute.
    fn is_constant_storage(&self) -> bool;
}

impl EvmStorageExt for StorageDecl {
    fn is_constant_storage(&self) -> bool {
        self.attrs.iter().any(|a| {
            a.namespace == "evm"
                && matches!(a.key.as_str(), evm_attrs::IS_CONSTANT | evm_attrs::IS_IMMUTABLE)
                && matches!(&a.value, AttrValue::Bool(true))
        })
    }
}
