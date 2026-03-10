//! Source IR (SIR) — a dialect-extensible intermediate representation
//! for smart contract analysis and verification.
//!
//! SIR is a language-neutral IR: a small, clean core grammar extended by
//! per-chain dialect definitions that plug in via four extension points
//! (`DialectExpr`, `DialectStmt`, `DialectMemberDecl`, `DialectType`).
//! The first dialect implemented is `evm` (covering Solidity and Vyper).

pub mod attrs;
pub mod cfg;
pub mod defs;
pub mod dialect;
pub mod exprs;
pub mod lits;
pub mod loc;
pub mod module;
pub mod spec;
pub mod stmts;
pub mod types;
pub mod utils;

// Re-exports for convenient access
pub use attrs::*;
pub use defs::*;
pub use dialect::*;
pub use exprs::*;
pub use lits::*;
pub use loc::*;
pub use module::*;
pub use spec::*;
pub use stmts::*;
pub use types::*;
pub use utils::{Fold, Map, Visit};
pub use utils::{export_module, print_ir};
