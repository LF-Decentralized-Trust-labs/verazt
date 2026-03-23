//! Canonical IR (CIR) — a normalized, structured intermediate representation.
//!
//! CIR sits between SIR and BIR in the compilation pipeline:
//!
//! ```text
//! SIR  (language-neutral, dialect-extensible)
//!   │
//!   ▼ sir::lower (scirs/src/sir/lower/)
//! CIR  (canonical, normalized, still structured)
//!   │
//!   ▼ cir::lower (scirs/src/cir/lower/)
//! BIR  (graph/SSA, analysis engine input)
//! ```
//!
//! CIR provides compile-time guarantees that its input is already normalized:
//! - Inheritance is resolved (no `parents` field)
//! - Modifiers are inlined into function bodies
//! - Named arguments are converted to positional
//! - Using-for directives are eliminated
//! - Expressions are flattened (call args are atoms)
//!
//! CIR reuses SIR types where there is no structural difference (`Type`, `Lit`,
//! `Attr`, `Span`, `FuncSpec`, dialect extension points).

pub mod defs;
pub mod exprs;
pub mod lower;
pub mod module;
pub mod stmts;
pub mod utils;
pub mod verifier;

// Re-exports for convenient access.
pub use defs::*;
pub use exprs::*;
pub use module::*;
pub use stmts::*;

// Re-export shared SIR types that CIR uses without change.
pub use crate::sir::{
    Attr, AttrValue, BinOp, FuncSpec, Lit, OverflowSemantics, Span, StorageRef, Type, TypeParam,
    UnOp,
};
pub use crate::sir::{DialectExpr, DialectMemberDecl, DialectStmt, DialectType};
