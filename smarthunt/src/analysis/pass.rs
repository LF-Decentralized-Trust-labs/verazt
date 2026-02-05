//! Core Pass Traits
//!
//! This module defines the core traits for passes in the analysis framework.

use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::analysis::context::AnalysisContext;
use std::fmt::{self, Display};
use thiserror::Error;

/// Error type for pass execution.
#[derive(Debug, Error)]
pub enum PassError {
    /// Pass execution failed.
    #[error("Pass '{0}' failed: {1}")]
    ExecutionFailed(String, String),

    /// Dependency not satisfied.
    #[error("Dependency not satisfied: pass '{0}' requires pass '{1}'")]
    DependencyNotSatisfied(String, String),

    /// Context missing required data.
    #[error("Context missing required data: {0}")]
    MissingData(String),

    /// IR not available but required.
    #[error("IR not available: pass '{0}' requires IR generation")]
    IrNotAvailable(String),

    /// Circular dependency detected.
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    /// Pass not found.
    #[error("Pass not found: {0}")]
    PassNotFound(String),

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
}

/// Result type for pass operations.
pub type PassResult<T> = Result<T, PassError>;

/// Base trait for all passes.
///
/// This trait defines the common interface for both analysis passes
/// and bug detection passes. All passes must be thread-safe (`Send + Sync`).
pub trait Pass: Send + Sync {
    /// Get the unique identifier for this pass.
    fn id(&self) -> PassId;

    /// Get a human-readable name for this pass.
    fn name(&self) -> &'static str;

    /// Get a description of what this pass does.
    fn description(&self) -> &'static str;

    /// Get the granularity level at which this pass operates.
    fn level(&self) -> PassLevel;

    /// Get the representation this pass operates on.
    fn representation(&self) -> PassRepresentation;

    /// Get the list of passes that must run before this one.
    fn dependencies(&self) -> Vec<PassId>;

    /// Get the list of passes that this pass invalidates (for future transformation passes).
    fn invalidates(&self) -> Vec<PassId> {
        vec![]
    }

    /// Check if this pass is enabled by default.
    fn enabled_by_default(&self) -> bool {
        true
    }
}

/// Trait for analysis passes.
///
/// Analysis passes collect information from the AST/IR and store
/// results in the `AnalysisContext`. They are read-only with respect
/// to the source representation.
pub trait AnalysisPass: Pass {
    /// Run the pass and update the analysis context.
    ///
    /// This method should:
    /// 1. Read from the AST/IR as needed
    /// 2. Perform analysis
    /// 3. Store results in the context
    fn run(&self, context: &mut AnalysisContext) -> PassResult<()>;

    /// Check if this pass has already been run on the given context.
    fn is_completed(&self, context: &AnalysisContext) -> bool;
}

/// Metadata about a pass execution.
#[derive(Debug, Clone)]
pub struct PassExecutionInfo {
    /// Pass identifier.
    pub pass_id: PassId,
    /// Pass name.
    pub name: String,
    /// Execution duration.
    pub duration: std::time::Duration,
    /// Whether execution succeeded.
    pub success: bool,
    /// Error message if failed.
    pub error: Option<String>,
}

impl Display for PassExecutionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.success {
            write!(
                f,
                "Pass {} ({}) completed in {:?}",
                self.name, self.pass_id, self.duration
            )
        } else {
            write!(
                f,
                "Pass {} ({}) failed: {}",
                self.name,
                self.pass_id,
                self.error.as_deref().unwrap_or("unknown error")
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pass_error_display() {
        let err = PassError::ExecutionFailed(
            "test-pass".to_string(),
            "something went wrong".to_string(),
        );
        assert!(err.to_string().contains("test-pass"));
        assert!(err.to_string().contains("something went wrong"));
    }
}
