pub mod cfg;
pub mod structural;

// Re-export CFG types
pub use cfg::{BasicBlock, BasicBlockId, CfgPass, ControlFlowGraph, Terminator};
