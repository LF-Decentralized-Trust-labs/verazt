//! Core Pass Traits
//!
//! This module defines the core traits for passes in the analysis framework.

use crate::context::{AnalysisContext, ContextKey, ErasedArtifact};
use crate::passes::base::meta::{PassLevel, PassRepresentation};
use std::any::TypeId;
use std::fmt::{self, Display};
use thiserror::Error;

/// Error type for pass execution.
#[derive(Debug, Error)]
pub enum PassError {
    /// Circular dependency detected, through the named passes.
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    /// Pass execution failed.
    #[error("Pass \'{0}\' failed: {1}")]
    ExecutionFailed(String, String),

    /// A pass or detector depends on a pass that is not registered.
    #[error("\'{0}\' depends on a pass that is not registered")]
    UnregisteredDependency(String),
}

/// Result type for pass operations.
pub type PassResult<T> = Result<T, PassError>;

/// Base trait for all passes.
///
/// This trait defines the common interface for both analysis passes
/// and bug detection passes. All passes must be thread-safe (`Send + Sync`).
///
/// Pass identity is based on `std::any::TypeId`: each concrete type
/// gets a compiler-guaranteed unique ID with zero maintenance overhead.
pub trait Pass: Send + Sync + 'static {
    /// Get the unique identifier for this pass.
    fn id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    /// Get a human-readable name for this pass.
    fn name(&self) -> &'static str;

    /// Get a description of what this pass does.
    fn description(&self) -> &'static str;

    /// Get the granularity level at which this pass operates.
    fn level(&self) -> PassLevel;

    /// Get the representation this pass operates on.
    fn representation(&self) -> PassRepresentation;

    /// Get the list of passes that must run before this one.
    fn dependencies(&self) -> Vec<TypeId>;
}

/// Trait for analysis passes.
///
/// An analysis pass computes one artifact from the IR and from the
/// artifacts of the passes it depends on. It only reads the context: the
/// executor stores the returned artifact under `Self::Artifact`, so that
/// the passes of one dependency level can run in parallel.
pub trait AnalysisPass: Pass {
    /// The key under which the context stores the artifact of the pass.
    type Artifact: ContextKey;

    /// Compute the artifact of the pass from `context`, which holds the
    /// artifacts of every pass this one depends on.
    fn run(&self, context: &AnalysisContext) -> PassResult<<Self::Artifact as ContextKey>::Value>;
}

/// Object-safe form of [`AnalysisPass`], through which the pass manager
/// holds and runs passes producing different artifact types. Implemented
/// for every analysis pass.
pub trait ErasedAnalysisPass: Pass {
    /// Run the pass, returning its artifact for the executor to store.
    fn run_erased(&self, context: &AnalysisContext) -> PassResult<ErasedArtifact>;
}

impl<P: AnalysisPass> ErasedAnalysisPass for P {
    fn run_erased(&self, context: &AnalysisContext) -> PassResult<ErasedArtifact> {
        self.run(context).map(ErasedArtifact::new::<P::Artifact>)
    }
}

/// Metadata about a pass execution.
#[derive(Debug, Clone)]
pub struct PassExecutionInfo {
    /// Pass identifier.
    pub pass_id: TypeId,
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
            write!(f, "Pass {} completed in {:?}", self.name, self.duration)
        } else {
            write!(
                f,
                "Pass {} failed: {}",
                self.name,
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
