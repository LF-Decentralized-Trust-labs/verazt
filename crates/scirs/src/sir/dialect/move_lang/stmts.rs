//! Move dialect statements with enum-of-structs pattern.

use crate::sir::exprs::Expr;
use common::loc::Loc;
use std::fmt::{self, Display};

// ═══════════════════════════════════════════════════════════════════
// Struct definitions for each MoveStmt variant
// ═══════════════════════════════════════════════════════════════════

/// `abort code` — terminate with error code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveAbort {
    pub expr: Box<Expr>,
    pub loc: Loc,
}

/// `spec { ... }` — spec block embedded in function body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveSpecBlock {
    pub assertions: Vec<Expr>,
    pub loc: Loc,
}

// ═══════════════════════════════════════════════════════════════════
// MoveStmt enum (enum-of-structs)
// ═══════════════════════════════════════════════════════════════════

/// Move-specific statements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveStmt {
    Abort(MoveAbort),
    SpecBlock(MoveSpecBlock),
}

impl Display for MoveStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoveStmt::Abort(e) => write!(f, "abort {};", e.expr),
            MoveStmt::SpecBlock(e) => {
                writeln!(f, "spec {{")?;
                for a in &e.assertions {
                    writeln!(f, "    {a};")?;
                }
                write!(f, "}}")
            }
        }
    }
}
