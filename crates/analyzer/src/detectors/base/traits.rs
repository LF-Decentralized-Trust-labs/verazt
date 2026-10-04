//! Bug Detector Trait
//!
//! Extends the analysis framework's Pass trait with vulnerability detection
//! capabilities.

use super::id::DetectorId;
use super::meta::{DetectorMeta, Target};
use crate::context::AnalysisContext;
use crate::passes::base::Pass;
use bugs::bug::Bug;

/// Confidence level for a detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConfidenceLevel {
    /// Low confidence - possible issue, needs careful review.
    Low,
    /// Medium confidence - likely issue but may need manual review.
    Medium,
    /// High confidence - very likely to be a real issue.
    High,
}

impl std::fmt::Display for ConfidenceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfidenceLevel::High => write!(f, "High"),
            ConfidenceLevel::Medium => write!(f, "Medium"),
            ConfidenceLevel::Low => write!(f, "Low"),
        }
    }
}

/// Result type for detector operations.
pub type DetectorResult<T> = Result<T, DetectorError>;

/// Error type for detector execution.
#[derive(Debug, thiserror::Error)]
pub enum DetectorError {
    #[error("Detector '{0}' failed: {1}")]
    ExecutionFailed(String, String),

    #[error("Missing required analysis: {0}")]
    MissingAnalysis(String),

    #[error("Analysis pass error: {0}")]
    AnalysisError(#[from] crate::passes::base::PassError),
}

/// Trait for bug detection passes.
///
/// This extends the base Pass trait from the analysis crate with
/// vulnerability detection capabilities. Each detector:
///
/// Detectors declare their analysis dependencies through [`Pass`]; `detect`
/// runs once those analyses have stored their artifacts in the context.
pub trait BugDetectionPass: Pass {
    /// The detector's identity, classification, and guidance.
    fn meta(&self) -> &'static DetectorMeta;

    /// Run detection and return found bugs.
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>>;

    /// Detectors whose findings this one subsumes. When both would run, the
    /// pipeline drops the superseded ones unless they are explicitly enabled.
    fn supersedes(&self) -> Vec<DetectorId> {
        vec![]
    }

    /// Whether the detector applies to `context`: by default, when the input
    /// language runs on the detector's target platform.
    fn is_enabled(&self, context: &AnalysisContext) -> bool {
        self.meta().target == Target::of(context.input_language)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_level_ordering() {
        assert!(ConfidenceLevel::High > ConfidenceLevel::Medium);
        assert!(ConfidenceLevel::Medium > ConfidenceLevel::Low);
    }

    #[test]
    fn test_confidence_level_display() {
        assert_eq!(format!("{}", ConfidenceLevel::High), "High");
        assert_eq!(format!("{}", ConfidenceLevel::Medium), "Medium");
        assert_eq!(format!("{}", ConfidenceLevel::Low), "Low");
    }
}
