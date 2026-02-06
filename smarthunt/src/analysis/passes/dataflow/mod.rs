//! Data Flow Analysis Framework
//!
//! This module provides a generic, reusable infrastructure for implementing
//! various data flow analyses using the worklist algorithm and lattice-based
//! abstract interpretation.
//!
//! # Architecture
//!
//! The framework consists of:
//! - **Core Framework**: Generic solver and trait definitions
//! - **Specific Analyses**: Concrete implementations (taint, def-use, liveness)
//! - **Integration**: PassManager integration and artifact storage
//!
//! # Usage
//!
//! To implement a new data flow analysis:
//! 1. Define a `DataFlowFact` (lattice element)
//! 2. Implement a `TransferFunction`
//! 3. Create an `AnalysisPass` that uses `DataFlowSolver`
//! 4. Register the pass with PassManager

pub mod framework;
pub mod taint;
pub mod def_use;
pub mod liveness;
pub mod state_mutation;

// Re-export core framework types
pub use framework::{
    DataFlowFact, TransferFunction, AnalysisDirection,
    DataFlowSolver, DataFlowResult, DataFlowError,
};

// Re-export specific analyses
pub use taint::{TaintAnalysisPass, TaintMap, TaintSource, TaintSink, TaintStatus};
pub use def_use::{DefUseChainPass, DefUseChain, Definition, Use};
pub use liveness::{LivenessAnalysisPass, LiveSet};
pub use state_mutation::{StateMutationPass, StateMutation, StateWrite, StateRead};
