//! Dialect extension system for SIR.
//!
//! Dialects are the mechanism by which chain-specific constructs
//! (EVM opcodes, Move resources, Anchor accounts) plug into SIR
//! without polluting the core grammar.
//!
//! Currently implemented dialects:
//! - `evm` — Solidity, Vyper, Fe (all EVM-targeting languages)
//! - `anchor` — Anchor framework / Solana
//! - `move` — Move language (Aptos, Sui)
//! - `spec` — Formal verification (cross-chain)

pub mod anchor;
pub mod evm;
pub mod move_lang;
pub mod spec_dialect;

use std::fmt::{self, Display};

// ─── Dialect Enums ─────────────────────────────────────────────────
// Using enums (rather than trait objects) for ergonomic Rust derive support.
// New dialects are added by extending these enums.

/// A dialect-specific expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialectExpr {
    Evm(evm::EvmExpr),
    Anchor(anchor::AnchorExpr),
    Move(move_lang::MoveExpr),
}

/// A dialect-specific statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialectStmt {
    Evm(evm::EvmStmt),
    Anchor(anchor::AnchorStmt),
    Move(move_lang::MoveStmt),
}

/// A dialect-specific member declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialectMemberDecl {
    Evm(evm::EvmMemberDecl),
    Anchor(anchor::AnchorMemberDecl),
    Move(move_lang::MoveMemberDecl),
}

/// A dialect-specific type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DialectType {
    Evm(evm::EvmType),
    Anchor(anchor::AnchorType),
    Move(move_lang::MoveType),
    Spec(spec_dialect::SpecType),
}

/// Dialect-level declarations (future use).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialectDecl {
    Evm(evm::EvmDecl),
    Anchor(anchor::AnchorDecl),
    Move(move_lang::MoveDecl),
}

impl DialectExpr {
    pub fn typ(&self) -> crate::sir::types::Type {
        match self {
            DialectExpr::Evm(e) => e.typ(),
            DialectExpr::Anchor(_) => crate::sir::types::Type::None,
            DialectExpr::Move(_) => crate::sir::types::Type::None,
        }
    }
}

// ─── Display implementations ──────────────────────────────────────

impl Display for DialectExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DialectExpr::Evm(e) => write!(f, "{e}"),
            DialectExpr::Anchor(e) => write!(f, "{e}"),
            DialectExpr::Move(e) => write!(f, "{e}"),
        }
    }
}

impl Display for DialectStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DialectStmt::Evm(s) => write!(f, "{s}"),
            DialectStmt::Anchor(s) => write!(f, "{s}"),
            DialectStmt::Move(s) => write!(f, "{s}"),
        }
    }
}

impl Display for DialectMemberDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DialectMemberDecl::Evm(d) => write!(f, "{d}"),
            DialectMemberDecl::Anchor(d) => write!(f, "{d}"),
            DialectMemberDecl::Move(d) => write!(f, "{d}"),
        }
    }
}

impl Display for DialectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DialectType::Evm(t) => write!(f, "{t}"),
            DialectType::Anchor(t) => write!(f, "{t}"),
            DialectType::Move(t) => write!(f, "{t}"),
            DialectType::Spec(t) => write!(f, "{t}"),
        }
    }
}

impl Display for DialectDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DialectDecl::Evm(d) => write!(f, "{d}"),
            DialectDecl::Anchor(d) => write!(f, "{d}"),
            DialectDecl::Move(d) => write!(f, "{d}"),
        }
    }
}
