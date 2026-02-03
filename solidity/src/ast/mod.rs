//! Solidity AST.

pub mod block;
pub mod builtins;
pub mod defs;
pub mod directives;
pub mod exprs;
pub mod ident;
pub mod lits;
pub mod source_unit;
pub mod specs;
pub mod stmts;
pub mod types;
pub mod utils;
pub mod yul;

pub use block::*;
pub use builtins::*;
pub use defs::*;
pub use directives::*;
pub use exprs::*;
pub use ident::*;
pub use lits::*;
pub use source_unit::*;
pub use specs::*;
pub use stmts::*;
pub use types::*;

/// Trait to export AST elements to compilable Solidity code.
pub trait Export {
    fn export(&self, print_data_loc: bool) -> String;
}
