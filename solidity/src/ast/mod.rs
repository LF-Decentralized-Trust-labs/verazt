//! Solidity AST.

pub mod block;
pub mod builtins;
pub mod definitions;
pub mod directives;
pub mod expressions;
pub mod identifier;
pub mod literals;
pub mod source_unit;
pub mod specifiers;
pub mod statements;
pub mod types;
pub mod utils;

pub use block::*;
pub use builtins::*;
pub use definitions::*;
pub use directives::*;
pub use expressions::*;
pub use identifier::*;
pub use literals::*;
pub use source_unit::*;
pub use specifiers::*;
pub use statements::*;
pub use types::*;

/// Trait to export AST elements to compilable Solidity code.
pub trait Export {
    fn export(&self, print_data_loc: bool) -> String;
}
