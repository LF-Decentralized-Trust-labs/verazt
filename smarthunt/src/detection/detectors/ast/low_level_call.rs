//! Low-Level Call Detector
//!
//! Detects usage of low-level calls like call, delegatecall, staticcall.

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::detection::pass::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::analysis::context::AnalysisContext;

/// Detector for low-level calls.
pub struct LowLevelCallDetector;

impl LowLevelCallDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LowLevelCallDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for LowLevelCallDetector {
    fn id(&self) -> PassId {
        PassId::LowLevelCall
    }

    fn name(&self) -> &'static str {
        "Low-Level Calls"
    }

    fn description(&self) -> &'static str {
        "Detects usage of low-level calls that may be dangerous."
    }

    fn level(&self) -> PassLevel {
        PassLevel::Expression
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ast
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::SymbolTable]
    }
}

impl BugDetectionPass for LowLevelCallDetector {
    fn detect(&self, _context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        // TODO: Implement low-level call detection
        Ok(vec![])
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Medium
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::Medium
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![]
    }
}
