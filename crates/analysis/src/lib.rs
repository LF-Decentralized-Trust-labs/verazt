//! Analysis Framework for Solidity Code
//!
//! # Architecture
//!
//! The analysis framework provides:
//! - **Pass Infrastructure**: Base traits for analysis passes
//! - **PassManager**: Orchestrates pass registration, scheduling, and execution
//! - **AnalysisContext**: Central storage for AST, IR, and analysis artifacts
//! - **Dependency Resolution**: Automatic scheduling based on pass dependencies
//! - **Abstract Interpretation**: Generic lattice framework, worklist-based
//!   fixpoint solver, and built-in dataflow analyses (reaching defs, liveness,
//!   taint, etc.) shared across crates.
//!
//! # Usage
//!
//! ```ignore
//! use analysis::{PassManager, AnalysisContext, PassManagerConfig};
//!
//! let mut manager = PassManager::new(PassManagerConfig::default());
//! // Register passes...
//!
//! let mut context = AnalysisContext::new(source_units, Default::default());
//! let report = manager.run(&mut context)?;
//! ```

pub mod context;
pub mod dependency;
pub mod executor;
pub mod manager;
pub mod pass;
pub mod pass_id;
pub mod pass_level;
pub mod pass_representation;
pub mod scheduler;

// Analysis domains
pub mod air;
pub mod ast;
pub mod sir;

// Abstract Interpretation framework (shared across crates)
pub mod absint;

// Re-exports for convenient access
pub use context::{AnalysisConfig, AnalysisContext, AnalysisStats, InputLanguage};
pub use dependency::DependencyGraph;
pub use executor::PassExecutor;
pub use manager::{AnalysisReport, PassManager, PassManagerConfig};
pub use pass::{AnalysisPass, Pass, PassError, PassResult};
pub use pass_id::PassId;
pub use pass_level::PassLevel;
pub use pass_representation::PassRepresentation;
pub use scheduler::PassScheduler;

// Re-export concrete passes
pub use ast::{
    CallGraph, CallGraphExt, CallGraphPass, CallSite, FunctionId, InheritanceGraph,
    InheritanceGraphExt, InheritanceGraphPass, ModifierAnalysis, ModifierAnalysisExt,
    ModifierAnalysisPass, ModifierInfo, SymbolTable, SymbolTableExt, SymbolTablePass, TypeIndex,
    TypeIndexExt, TypeIndexPass, TypeInfo,
};

// Re-export SIR/CFG passes
pub use sir::{BasicBlock, BasicBlockId, CfgPass, ControlFlowGraph, Terminator};

// Re-export AIR passes
pub use air::{
    AIRAccessControlPass, AIRArithmeticPass, AIRGenerationPass, AIRTaintPropagationPass,
};

// Re-export absint framework
pub use absint::{
    FlatLattice, Lattice, MapLattice, PowerSetLattice, ProductLattice,
    DataFlowResult, DataFlowSolver, Direction, Transfer,
    VarId, VarScope,
};
pub use absint::analyses::{
    DefUseChainsPass, Use,
    LiveVarsTransfer, LivenessPass,
    Definition, ReachingDefsPass, ReachingDefsTransfer,
    AccessKind, StateAccess, StateMutationPass,
    TaintAnalysisPass, TaintSink, TaintSource,
};
