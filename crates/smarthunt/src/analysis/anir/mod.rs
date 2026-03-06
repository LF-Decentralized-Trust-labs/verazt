//! IR-based Analysis Passes
//!
//! These passes operate on the Intermediate Representation (IR) of Solidity
//! code. IR provides a lower-level, more structured view suitable for control
//! flow and data flow analysis.

pub mod cfg;


// ANIR generation and analysis passes
pub mod generation;
pub mod taint;
pub mod access_control;
pub mod arithmetic;

// Re-export pass implementations
pub use cfg::{BasicBlock, BasicBlockId, CfgPass, ControlFlowGraph, Terminator};

// Re-export ANIR passes
pub use generation::AnirGenerationPass;
pub use taint::AnirTaintPropagationPass;
pub use access_control::AnirAccessControlPass;
pub use arithmetic::AnirArithmeticPass;
