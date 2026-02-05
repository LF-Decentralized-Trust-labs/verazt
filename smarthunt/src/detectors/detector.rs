//! Detector trait definition.
//!
//! This module defines the core detector trait that all detectors implement.

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::detectors::context::AnalysisContext;
use crate::detectors::ConfidenceLevel;

/// Trait for bug detectors.
///
/// All detectors implement this trait to provide a unified interface
/// for bug detection.
pub trait Detector: Send + Sync {
    /// Unique detector identifier.
    fn id(&self) -> &'static str;

    /// Human-readable detector name.
    fn name(&self) -> &'static str;

    /// Detailed description of what this detector finds.
    fn description(&self) -> &'static str;

    /// Bug category (Vulnerability, Refactoring, Optimization).
    fn bug_kind(&self) -> BugKind;

    /// Risk level (Critical, High, Medium, Low, No/Informational).
    fn risk_level(&self) -> RiskLevel;

    /// Confidence level (High, Medium, Low).
    fn confidence(&self) -> ConfidenceLevel;

    /// Associated CWE IDs.
    fn cwe_ids(&self) -> Vec<usize>;

    /// Associated SWC IDs (Smart Contract Weakness Classification).
    fn swc_ids(&self) -> Vec<usize>;

    /// Run the detector and return findings.
    fn detect(&self, context: &AnalysisContext) -> Vec<Bug>;

    /// Get a recommendation for fixing the issue.
    fn recommendation(&self) -> &'static str {
        ""
    }

    /// Get references/documentation links.
    fn references(&self) -> Vec<&'static str> {
        vec![]
    }
}

/// Helper to create a Bug from detector info.
pub fn create_bug(
    detector: &dyn Detector,
    description: Option<&str>,
    loc: solidity::ast::Loc,
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
