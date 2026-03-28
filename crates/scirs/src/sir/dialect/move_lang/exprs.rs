//! Move dialect expressions with enum-of-structs pattern.

use crate::sir::exprs::Expr;
use crate::sir::types::Type;
use common::loc::Loc;
use std::fmt::{self, Display};

// ═══════════════════════════════════════════════════════════════════
// Struct definitions for each MoveExpr variant
// ═══════════════════════════════════════════════════════════════════

/// `move.borrow_global<T>(addr)` — immutable global storage borrow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveBorrowGlobal {
    pub ty: Type,
    pub addr: Box<Expr>,
    pub loc: Loc,
}

/// `move.borrow_global_mut<T>(addr)` — mutable global storage borrow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveBorrowGlobalMut {
    pub ty: Type,
    pub addr: Box<Expr>,
    pub loc: Loc,
}

/// `move.exists<T>(addr)` — check if resource exists at address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveExists {
    pub ty: Type,
    pub addr: Box<Expr>,
    pub loc: Loc,
}

/// `move.signer_address(signer)` — get address from signer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveSignerAddress {
    pub expr: Box<Expr>,
    pub loc: Loc,
}

/// `move.move_to(resource, signer)` — publish resource to signer's address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveMoveTo {
    pub resource: Box<Expr>,
    pub signer: Box<Expr>,
    pub loc: Loc,
}

/// `move.move_from<T>(addr)` — remove resource from address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveMoveFrom {
    pub ty: Type,
    pub addr: Box<Expr>,
    pub loc: Loc,
}

/// `move.write_ref(ref, value)` — write through a mutable reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveWriteRef {
    pub reference: Box<Expr>,
    pub value: Box<Expr>,
    pub loc: Loc,
}

/// `move.ghost_var(name)` — spec-only ghost variable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveGhostVar {
    pub name: String,
    pub loc: Loc,
}

// ═══════════════════════════════════════════════════════════════════
// MoveExpr enum (enum-of-structs)
// ═══════════════════════════════════════════════════════════════════

/// Move-specific expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveExpr {
    BorrowGlobal(MoveBorrowGlobal),
    BorrowGlobalMut(MoveBorrowGlobalMut),
    Exists(MoveExists),
    SignerAddress(MoveSignerAddress),
    MoveTo(MoveMoveTo),
    MoveFrom(MoveMoveFrom),
    WriteRef(MoveWriteRef),
    GhostVar(MoveGhostVar),
}

impl Display for MoveExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoveExpr::BorrowGlobal(e) => {
                write!(f, "move.borrow_global<{}>({e})", e.ty, e = e.addr)
            }
            MoveExpr::BorrowGlobalMut(e) => {
                write!(f, "move.borrow_global_mut<{}>({e})", e.ty, e = e.addr)
            }
            MoveExpr::Exists(e) => write!(f, "move.exists<{}>({})", e.ty, e.addr),
            MoveExpr::SignerAddress(e) => write!(f, "move.signer_address({})", e.expr),
            MoveExpr::MoveTo(e) => {
                write!(f, "move.move_to({}, {})", e.resource, e.signer)
            }
            MoveExpr::MoveFrom(e) => {
                write!(f, "move.move_from<{}>({})", e.ty, e.addr)
            }
            MoveExpr::WriteRef(e) => {
                write!(f, "move.write_ref({}, {})", e.reference, e.value)
            }
            MoveExpr::GhostVar(e) => write!(f, "move.ghost_var({})", e.name),
        }
    }
}
