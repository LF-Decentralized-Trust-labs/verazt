//! EVM dialect for SIR.
//!
//! Covers Solidity and Vyper constructs that are specific to the EVM:
//! address types, msg/block/tx globals, storage ops, events, modifiers,
//! and inline assembly.

mod decls;
mod exprs;
mod ext;
mod stmts;
mod types;

pub use decls::*;
pub use exprs::*;
pub use ext::*;
pub use stmts::*;
pub use types::*;
