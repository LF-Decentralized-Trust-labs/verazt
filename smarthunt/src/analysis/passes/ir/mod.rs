//! IR-based Analysis Passes
//!
//! These passes operate on the Intermediate Representation (IR) of Solidity code.
//! IR provides a lower-level, more structured view suitable for control flow
//! and data flow analysis.

pub mod cfg;

// Re-export pass implementations
pub use cfg::{CfgPass, ControlFlowGraph, BasicBlock, BasicBlockId, Terminator};
