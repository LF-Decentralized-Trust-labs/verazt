//! Anchor dialect statements with enum-of-structs pattern.

use crate::sir::exprs::Expr;
use common::loc::Loc;
use std::fmt::{self, Display};

// ═══════════════════════════════════════════════════════════════════
// Struct definitions for each AnchorStmt variant
// ═══════════════════════════════════════════════════════════════════

/// `anchor.emit_event` — event emission via Anchor's `emit!` macro.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorEmitEvent {
    pub event: String,
    pub fields: Vec<(String, Expr)>,
    pub loc: Loc,
}

// ═══════════════════════════════════════════════════════════════════
// AnchorStmt enum (enum-of-structs)
// ═══════════════════════════════════════════════════════════════════

/// Anchor-specific statements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnchorStmt {
    EmitEvent(AnchorEmitEvent),
}

impl Display for AnchorStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnchorStmt::EmitEvent(e) => {
                let fs: Vec<_> = e.fields.iter().map(|(k, v)| format!("{k}: {v}")).collect();
                write!(f, "emit!({} {{ {} }});", e.event, fs.join(", "))
            }
        }
    }
}
