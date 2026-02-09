//! Analysis Framework for Solidity Code
//!
//! **Migration Note**: This module was migrated from `solidity::analysis`
//! to provide SmartHunt with a self-contained analysis infrastructure.
//!
//! # Architecture
//!
//! The analysis framework provides:
//! - **Pass Infrastructure**: Base traits for analysis passes
//! - **PassManager**: Orchestrates pass registration, scheduling, and execution
//! - **AnalysisContext**: Central storage for AST, IR, and analysis artifacts
//! - **Dependency Resolution**: Automatic scheduling based on pass dependencies
//!
//! # Usage
//!
//! ```ignore
//! use smarthunt::analysis::{PassManager, AnalysisContext, PassManagerConfig};
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

// Analysis pass implementations
pub mod passes;

// Re-exports for convenient access
pub use context::{AnalysisConfig, AnalysisContext, AnalysisStats};
pub use dependency::DependencyGraph;
pub use executor::PassExecutor;
pub use manager::{AnalysisReport, PassManager, PassManagerConfig};
pub use pass::{AnalysisPass, Pass, PassError, PassResult};
pub use pass_id::PassId;
pub use pass_level::PassLevel;
pub use pass_representation::PassRepresentation;
pub use scheduler::PassScheduler;

// Re-export concrete passes
pub use passes::ast::{
    CallGraph, CallGraphExt, CallGraphPass, CallSite, FunctionId, InheritanceGraph,
    InheritanceGraphExt, InheritanceGraphPass, ModifierAnalysis, ModifierAnalysisExt,
    ModifierAnalysisPass, ModifierInfo, SymbolTable, SymbolTableExt, SymbolTablePass, TypeIndex,
    TypeIndexExt, TypeIndexPass, TypeInfo,
};

// Re-export IR passes
pub use passes::ir::{BasicBlock, BasicBlockId, CfgPass, ControlFlowGraph, Terminator};
