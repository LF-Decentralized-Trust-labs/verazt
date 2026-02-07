//! Unchecked Call Detector
//!
//! Detects low-level calls whose return values are not checked.

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::detection::pass::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::analysis::context::AnalysisContext;

/// Detector for unchecked call returns.
pub struct UncheckedCallDetector;

impl UncheckedCallDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for UncheckedCallDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for UncheckedCallDetector {
    fn id(&self) -> PassId {
        PassId::UncheckedCall
    }

    fn name(&self) -> &'static str {
        "Unchecked Call Return"
    }

    fn description(&self) -> &'static str {
        "Detects low-level calls whose return values are not checked."
    }

    fn level(&self) -> PassLevel {
        PassLevel::Statement
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ast
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::SymbolTable]
    }
}

impl BugDetectionPass for UncheckedCallDetector {
    fn detect(&self, _context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        // TODO: Implement unchecked call detection
        Ok(vec![])
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Medium
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::High
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![252] // CWE-252: Unchecked Return Value
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![104] // SWC-104: Unchecked Call Return Value
    }

    fn recommendation(&self) -> &'static str {
        "Ensure the return value of the low-level call is checked."
    }
}
