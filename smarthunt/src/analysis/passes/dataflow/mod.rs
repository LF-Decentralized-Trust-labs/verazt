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

pub mod def_use;
pub mod framework;
pub mod liveness;
pub mod state_mutation;
pub mod taint;

// Re-export core framework types
pub use framework::{
    AnalysisDirection, DataFlowError, DataFlowFact, DataFlowResult, DataFlowSolver,
    TransferFunction,
};

// Re-export specific analyses
pub use def_use::{DefUseChain, DefUseChainPass, Definition, Use};
pub use liveness::{LiveSet, LivenessAnalysisPass};
pub use state_mutation::{StateMutation, StateMutationPass, StateRead, StateWrite};
pub use taint::{TaintAnalysisPass, TaintMap, TaintSink, TaintSource, TaintStatus};
