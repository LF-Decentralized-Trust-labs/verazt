//! Dialect extension system for CIR.
//!
//! Dialects are the mechanism by which chain-specific constructs
//! (EVM opcodes, Move resources, Anchor accounts) plug into CIR
//! without polluting the core grammar.

pub mod evm;

use std::fmt::{self, Display};

// ─── Dialect Enums ─────────────────────────────────────────────────
// Using enums (rather than trait objects) for ergonomic Rust derive support.
// New dialects are added by extending these enums.

/// A dialect-specific expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DialectExpr {
    Evm(evm::EvmExpr),
}

/// A dialect-specific statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialectStmt {
    Evm(evm::EvmStmt),
}

/// A dialect-specific member declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialectMemberDecl {
    Evm(evm::EvmMemberDecl),
}

/// A dialect-specific type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DialectType {
    Evm(evm::EvmType),
}

/// Dialect-level declarations (future use).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialectDecl {
    Evm(evm::EvmDecl),
}

// ─── Display implementations ──────────────────────────────────────

impl Display for DialectExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DialectExpr::Evm(e) => write!(f, "{e}"),
        }
    }
}

impl Display for DialectStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DialectStmt::Evm(s) => write!(f, "{s}"),
        }
    }
}

impl Display for DialectMemberDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DialectMemberDecl::Evm(d) => write!(f, "{d}"),
        }
    }
}

impl Display for DialectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DialectType::Evm(t) => write!(f, "{t}"),
        }
    }
}

impl Display for DialectDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DialectDecl::Evm(d) => write!(f, "{d}"),
        }
    }
}
