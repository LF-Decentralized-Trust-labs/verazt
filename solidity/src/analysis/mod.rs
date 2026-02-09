//! Analysis Framework for Solidity Code
//!
//! This module provides a reusable analysis infrastructure for Solidity code,
//! inspired by LLVM's pass-based architecture. It supports:
//!
//! - **Pass Infrastructure**: Traits and types for defining analysis passes
//! - **Pass Manager**: Central orchestrator for scheduling and executing passes
//! - **Analysis Context**: Storage for AST, IR, and analysis artifacts
//! - **Parallel Execution**: Support for running independent passes
//!   concurrently
//!
//! # Architecture
//!
//! The analysis framework operates on two representations:
//! - **AST**: High-level source structure for pattern matching and semantic
//!   analysis
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

pub mod context;
pub mod dependency;
pub mod executor;
pub mod manager;
pub mod pass;
pub mod pass_id;
pub mod pass_level;
pub mod pass_representation;
pub mod passes;
pub mod scheduler;

// Re-exports for convenient access
pub use context::{AnalysisConfig, AnalysisContext};
pub use dependency::DependencyGraph;
pub use executor::PassExecutor;
pub use manager::{AnalysisReport, PassManager, PassManagerConfig};
pub use pass::{AnalysisPass, Pass, PassError, PassResult};
pub use pass_id::PassId;
pub use pass_level::PassLevel;
pub use pass_representation::PassRepresentation;
pub use scheduler::PassScheduler;

// Re-export passes
pub use passes::ast::{
    CallGraphPass, InheritanceGraphPass, ModifierAnalysisPass, SymbolTablePass, TypeIndexPass,
    call_graph::{CallGraph, CallGraphExt, CallSite},
    inheritance_graph::{InheritanceGraph, InheritanceGraphExt},
    modifier_analysis::{ModifierAnalysis, ModifierAnalysisExt, ModifierInfo},
    symbol_table::{FunctionId, SymbolTable, SymbolTableExt},
    type_index::{TypeIndex, TypeIndexExt, TypeInfo},
};
