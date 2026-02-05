//! Pass trait and PassId definitions.
//!
//! This module defines the core pass infrastructure for SmartHunt.

use crate::engine::context::AnalysisContext;
use std::fmt::{self, Display};
use thiserror::Error;

/// Error type for pass execution.
#[derive(Debug, Error)]
pub enum PassError {
    #[error("Pass '{0}' failed: {1}")]
    ExecutionFailed(String, String),
    
    #[error("Dependency not satisfied: pass '{0}' requires pass '{1}'")]
    DependencyNotSatisfied(String, String),
    
    #[error("Context missing required data: {0}")]
    MissingData(String),
}

/// Result type for pass operations.
pub type PassResult<T> = Result<T, PassError>;

/// Unique identifier for an analysis pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PassId {
    SymbolTable,
    TypeIndex,
    Cfg,
    CallGraph,
    DataFlow,
    StateMutation,
    AccessControl,
    LocalPatterns,
    LocalSemantic,
    StateVariables,
    Reentrancy,
    ExternalInteraction,
}

impl Display for PassId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PassId::SymbolTable => write!(f, "SymbolTable"),
            PassId::TypeIndex => write!(f, "TypeIndex"),
            PassId::Cfg => write!(f, "CFG"),
            PassId::CallGraph => write!(f, "CallGraph"),
            PassId::DataFlow => write!(f, "DataFlow"),
            PassId::StateMutation => write!(f, "StateMutation"),
            PassId::AccessControl => write!(f, "AccessControl"),
            PassId::LocalPatterns => write!(f, "LocalPatterns"),
            PassId::LocalSemantic => write!(f, "LocalSemantic"),
            PassId::StateVariables => write!(f, "StateVariables"),
            PassId::Reentrancy => write!(f, "Reentrancy"),
            PassId::ExternalInteraction => write!(f, "ExternalInteraction"),
        }
    }
}

/// Trait for analysis passes.
///
/// Each pass performs a specific analysis on the AST/IR and produces
/// artifacts that can be consumed by other passes or detectors.
pub trait AnalysisPass: Send + Sync {
    /// Get the unique identifier for this pass.
    fn id(&self) -> PassId;
    
    /// Get a human-readable name for this pass.
    fn name(&self) -> &'static str;
    
    /// Get a description of what this pass does.
    fn description(&self) -> &'static str;
    
    /// Get the list of passes that must run before this one.
    fn dependencies(&self) -> Vec<PassId>;
    
    /// Run the pass and update the analysis context.
    fn run(&self, context: &mut AnalysisContext) -> PassResult<()>;
    
    /// Check if this pass has already been run on the given context.
    fn is_completed(&self, context: &AnalysisContext) -> bool;
}

/// Phase of analysis execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnalysisPhase {
    /// Phase 1: Context building (sequential)
    ContextBuilding = 1,
    /// Phase 2: Local analysis (parallelizable)
    LocalAnalysis = 2,
    /// Phase 3: Inter-procedural analysis (partially parallelizable)
    InterProcedural = 3,
    /// Phase 4: Cross-contract analysis (limited parallelization)
    CrossContract = 4,
    /// Phase 5: Advanced semantic analysis (optional, expensive)
    AdvancedSemantic = 5,
}

impl Display for AnalysisPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnalysisPhase::ContextBuilding => write!(f, "Context Building"),
            AnalysisPhase::LocalAnalysis => write!(f, "Local Analysis"),
            AnalysisPhase::InterProcedural => write!(f, "Inter-procedural Analysis"),
            AnalysisPhase::CrossContract => write!(f, "Cross-contract Analysis"),
            AnalysisPhase::AdvancedSemantic => write!(f, "Advanced Semantic Analysis"),
        }
    }
}
