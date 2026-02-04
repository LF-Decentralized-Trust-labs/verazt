//! Solidity AST.

pub mod block;
pub mod builtins;
pub mod defs;
pub mod dirs;
pub mod exprs;
pub mod ident;
pub mod lits;
pub mod source_unit;
pub mod stmts;
pub mod types;
pub mod utils;
pub mod normalize;
pub mod yul;
pub mod loc;
pub mod name;

pub use block::*;
pub use builtins::*;
pub use defs::*;
pub use dirs::*;
pub use exprs::*;
pub use ident::*;
pub use lits::*;
pub use source_unit::*;
pub use stmts::*;
pub use types::*;
pub use loc::*;
pub use name::*;

/// Trait to export AST elements to compilable Solidity code.
pub trait Export {
    fn export(&self, print_data_loc: bool) -> String;
}
