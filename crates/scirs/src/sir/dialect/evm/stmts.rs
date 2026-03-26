//! EVM dialect statements with enum-of-structs pattern.

use crate::sir::exprs::Expr;
use crate::sir::stmts::Stmt;
use crate::sir::types::Type;
use common::loc::Loc;
use std::fmt::{self, Display};

// ═══════════════════════════════════════════════════════════════════
// Struct definitions for each EvmStmt variant
// ═══════════════════════════════════════════════════════════════════

/// `evm.emit_event(EventName, arg1, arg2, ...)`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmEmitEvent {
    pub event: String,
    pub args: Vec<Expr>,
    pub loc: Loc,
}

/// `evm.try_catch` — Solidity try/catch statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmTryCatch {
    pub guarded_expr: Expr,
    pub returns: Vec<(String, Type)>,
    pub body: Vec<Stmt>,
    pub catch_clauses: Vec<EvmCatchClause>,
    pub loc: Loc,
}

/// `_` — Modifier body injection point; replaced by `elim_modifiers` pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmPlaceholder {
    pub loc: Loc,
}

/// `selfdestruct(recipient)` — destroy contract and send funds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmSelfdestruct {
    pub recipient: Expr,
    pub loc: Loc,
}

// ═══════════════════════════════════════════════════════════════════
// EvmStmt enum (enum-of-structs)
// ═══════════════════════════════════════════════════════════════════

/// EVM-specific statements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvmStmt {
    EmitEvent(EvmEmitEvent),
    TryCatch(EvmTryCatch),
    Placeholder(EvmPlaceholder),
    Selfdestruct(EvmSelfdestruct),
}

/// A catch clause in a Solidity try/catch statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmCatchClause {
    pub error: Option<String>,
    pub params: Vec<(String, Type)>,
    pub body: Vec<Stmt>,
    pub loc: Loc,
}

impl Display for EvmStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvmStmt::EmitEvent(e) => {
                let a: Vec<_> = e.args.iter().map(|e| e.to_string()).collect();
                write!(f, "emit {}({});", e.event, a.join(", "))
            }
            EvmStmt::TryCatch(e) => {
                write!(f, "try {} {{ ... }}", e.guarded_expr)
            }
            EvmStmt::Placeholder(_) => write!(f, "_;"),
            EvmStmt::Selfdestruct(e) => write!(f, "selfdestruct({});", e.recipient),
        }
    }
}
