//! Analysis Framework for Solidity Code
//!
//! This module provides a reusable analysis infrastructure for Solidity code,
//! inspired by LLVM's pass-based architecture. It supports:
//!
//! - **Pass Infrastructure**: Traits and types for defining analysis passes
//! - **Pass Manager**: Central orchestrator for scheduling and executing passes
//! - **Analysis Context**: Storage for AST, IR, and analysis artifacts
//! - **Parallel Execution**: Support for running independent passes concurrently
//!
//! # Architecture
//!
//! The analysis framework operates on two representations:
//! - **AST**: High-level source structure for pattern matching and semantic analysis
//! - **IR**: Low-level normalized form for data flow and control flow analysis
//!
//! Passes can operate on either representation or both (hybrid passes).
//!
//! # Example
//!
//! ```ignore
//! use solidity::analysis::{PassManager, AnalysisContext, PassManagerConfig};
//!
//! let config = PassManagerConfig::default();
//! let mut manager = PassManager::new(config);
//!
//! // Register passes
//! manager.register_analysis_pass(Box::new(SymbolTablePass::new()));
//! manager.register_analysis_pass(Box::new(TypeIndexPass::new()));
//!
//! // Run all passes
//! let mut context = AnalysisContext::new(source_units, Default::default());
//! manager.run(&mut context)?;
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
pub mod passes;

// Re-exports for convenient access
pub use pass::{Pass, AnalysisPass, PassError, PassResult};
pub use pass_id::PassId;
pub use pass_level::PassLevel;
pub use pass_representation::PassRepresentation;
pub use context::{AnalysisContext, AnalysisConfig};
pub use manager::{PassManager, PassManagerConfig, AnalysisReport};
pub use scheduler::PassScheduler;
pub use executor::PassExecutor;
pub use dependency::DependencyGraph;

// Re-export passes
pub use passes::ast::{
    SymbolTablePass, TypeIndexPass, InheritanceGraphPass, CallGraphPass, ModifierAnalysisPass,
    symbol_table::{SymbolTable, FunctionId, SymbolTableExt},
    type_index::{TypeIndex, TypeInfo, TypeIndexExt},
    inheritance_graph::{InheritanceGraph, InheritanceGraphExt},
    call_graph::{CallGraph, CallSite, CallGraphExt},
    modifier_analysis::{ModifierAnalysis, ModifierInfo, ModifierAnalysisExt},
};
