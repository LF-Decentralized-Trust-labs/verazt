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

pub mod pass;
pub mod pass_id;
pub mod pass_level;
pub mod pass_representation;
pub mod context;
pub mod manager;
pub mod scheduler;
pub mod executor;
pub mod dependency;

// Analysis pass implementations
pub mod passes;

// Re-exports for convenient access
pub use pass::{Pass, AnalysisPass, PassError, PassResult};
pub use pass_id::PassId;
pub use pass_level::PassLevel;
pub use pass_representation::PassRepresentation;
pub use context::{AnalysisContext, AnalysisConfig, AnalysisStats};
pub use manager::{PassManager, PassManagerConfig, AnalysisReport};
pub use scheduler::PassScheduler;
pub use executor::PassExecutor;
pub use dependency::DependencyGraph;

// Re-export concrete passes
pub use passes::ast::{
    SymbolTablePass, SymbolTable, FunctionId, SymbolTableExt,
    TypeIndexPass, TypeIndex, TypeInfo, TypeIndexExt,
    CallGraphPass, CallGraph, CallSite, CallGraphExt,
    InheritanceGraphPass, InheritanceGraph, InheritanceGraphExt,
    ModifierAnalysisPass, ModifierAnalysis, ModifierInfo, ModifierAnalysisExt,
};
