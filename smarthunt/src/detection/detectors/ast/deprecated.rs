//! Deprecated Features Detector
//!
//! Detects usage of deprecated Solidity features.

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::detection::pass::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::analysis::context::AnalysisContext;

/// Detector for deprecated features.
pub struct DeprecatedDetector;

impl DeprecatedDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DeprecatedDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for DeprecatedDetector {
    fn id(&self) -> PassId {
        PassId::Deprecated
    }

    fn name(&self) -> &'static str {
        "Deprecated Features"
    }

    fn description(&self) -> &'static str {
        "Detects usage of deprecated Solidity constructs."
    }

    fn level(&self) -> PassLevel {
        PassLevel::Expression
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ast
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![]
    }
}

impl BugDetectionPass for DeprecatedDetector {
    fn detect(&self, _context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        // TODO: Implement deprecated features detection
        Ok(vec![])
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Refactoring
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Low
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::High
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![111] // SWC-111: Use of Deprecated Solidity Functions
    }

    fn recommendation(&self) -> &'static str {
        "Replace deprecated features with their modern equivalents."
    }
}
