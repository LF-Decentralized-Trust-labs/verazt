//! Bug Detection Pass Trait
//!
//! Extends the analysis framework's Pass trait with vulnerability detection capabilities.

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::analysis::pass::Pass;
use crate::analysis::context::AnalysisContext;
use solidity::ast::Loc;

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
    AnalysisError(#[from] crate::analysis::pass::PassError),
}

/// Trait for bug detection passes.
///
/// This extends the base Pass trait from the solidity crate with
/// vulnerability detection capabilities. Each detector:
///
/// - Operates on analysis artifacts from the AnalysisContext
/// - Produces zero or more Bug instances
/// - Has associated metadata (CWE, SWC, risk level, confidence)
///
/// # Implementation
///
/// Detectors should implement this trait along with the base Pass trait.
/// The `detect` method is called after all required analysis passes have run.
///
/// ```ignore
/// struct MyDetector;
///
/// impl Pass for MyDetector {
///     fn id(&self) -> PassId { PassId::TxOrigin }
///     fn name(&self) -> &'static str { "TX Origin" }
///     fn description(&self) -> &'static str { "Detects tx.origin usage" }
///     fn level(&self) -> PassLevel { PassLevel::Expression }
///     fn representation(&self) -> PassRepresentation { PassRepresentation::Ast }
///     fn dependencies(&self) -> Vec<PassId> { vec![PassId::SymbolTable] }
/// }
///
/// impl BugDetectionPass for MyDetector {
///     fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
///         // Detection logic here
///         Ok(vec![])
///     }
///     fn bug_kind(&self) -> BugKind { BugKind::Vulnerability }
///     fn risk_level(&self) -> RiskLevel { RiskLevel::High }
///     fn confidence(&self) -> ConfidenceLevel { ConfidenceLevel::High }
///     fn cwe_ids(&self) -> Vec<usize> { vec![477] }
///     fn swc_ids(&self) -> Vec<usize> { vec![115] }
/// }
/// ```
pub trait BugDetectionPass: Pass {
    /// Run detection and return found bugs.
    ///
    /// This is the main entry point for bug detection. The detector should
    /// examine the analysis artifacts in the context and return any bugs found.
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>>;

    /// The category of bug this detector finds.
    fn bug_kind(&self) -> BugKind;

    /// The risk level of bugs found by this detector.
    fn risk_level(&self) -> RiskLevel;

    /// The confidence level of this detector.
    fn confidence(&self) -> ConfidenceLevel;

    /// Associated CWE (Common Weakness Enumeration) IDs.
    fn cwe_ids(&self) -> Vec<usize>;

    /// Associated SWC (Smart Contract Weakness Classification) IDs.
    fn swc_ids(&self) -> Vec<usize>;

    /// Get a recommendation for fixing the issue.
    fn recommendation(&self) -> &'static str {
        ""
    }

    /// Get references/documentation links.
    fn references(&self) -> Vec<&'static str> {
        vec![]
    }

    /// Check if this detector is enabled for the given configuration.
    fn is_enabled(&self, _context: &AnalysisContext) -> bool {
        true
    }
}

/// Helper function to create a Bug from detector metadata.
///
/// This is a convenience function for detectors to create Bug instances
/// with consistent metadata.
pub fn create_bug(
    detector: &dyn BugDetectionPass,
    description: Option<&str>,
    loc: Loc,
) -> Bug {
    Bug::new(
        detector.name(),
        description,
        loc,
        detector.bug_kind(),
        detector.risk_level(),
        detector.cwe_ids(),
        detector.swc_ids(),
    )
}

/// Helper function to create a Bug with additional details.
pub fn create_bug_with_details(
    detector: &dyn BugDetectionPass,
    title: &str,
    description: Option<&str>,
    loc: Loc,
) -> Bug {
    Bug::new(
        title,
        description,
        loc,
        detector.bug_kind(),
        detector.risk_level(),
        detector.cwe_ids(),
        detector.swc_ids(),
    )
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
