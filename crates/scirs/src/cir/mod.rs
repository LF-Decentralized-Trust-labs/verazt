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
//! - Chain semantics are explicit: dialect constructs are mapped to shared
//!   forms (state `Load`/`Store`, resolved calls, `Env`, `Emit`) or to the
//!   typed remainder in `dialect`
//!
//! CIR reuses SIR types where there is no structural difference (`Type`, `Lit`,
//! `Attr`, `Loc`, `FuncSpec`, dialect declarations and types).

pub mod defs;
pub mod dialect;
pub mod exprs;
pub mod lower;
pub mod module;
pub mod stmts;
pub mod utils;
pub mod verifier;

// Re-exports for convenient access.
pub use defs::*;
pub use dialect::*;
pub use exprs::*;
pub use module::*;
pub use stmts::*;

// Re-export shared SIR types that CIR uses without change. Dialect
// declarations and types stay in SIR form; executable dialect constructs
// are canonicalized into `dialect`.
pub use crate::sir::{
    Attr, AttrValue, BinOp, FuncSpec, Lit, Loc, OverflowSemantics, StorageRef, Type, TypeParam,
    UnOp,
};
pub use crate::sir::{DialectMemberDecl, DialectType};
