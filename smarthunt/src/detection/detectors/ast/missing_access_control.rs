//! Missing Access Control Detector (Stub)
//!
//! Detects public/external functions that lack proper access control.
//!
//! NOTE: This is currently a stub implementation. Full access control detection
//! requires modifier analysis and state mutation tracking.

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::detection::pass::{BugDetectionPass, ConfidenceLevel, DetectorResult};

/// Detector for missing access control.
#[derive(Debug, Default)]
pub struct MissingAccessControlDetector;

impl MissingAccessControlDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for MissingAccessControlDetector {
    fn id(&self) -> PassId {
        PassId::MissingAccessControl
    }

    fn name(&self) -> &'static str {
        "Missing Access Control"
    }

    fn description(&self) -> &'static str {
        "Detects functions lacking proper access control (requires modifier analysis)"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ast
    }

    fn dependencies(&self) -> Vec<PassId> {
        // Will require: ModifierAnalysis, StateAccess
        vec![]
    }
}

impl BugDetectionPass for MissingAccessControlDetector {
    fn detect(&self, _context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        // TODO: Implement access control detection
        // This requires:
        // 1. Modifier analysis to understand access restrictions
        // 2. State mutation analysis to find sensitive operations
        // 3. Function visibility and usage analysis
        Ok(vec![])
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::High
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::Medium
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![284] // CWE-284: Improper Access Control
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![105, 106] // SWC-105: Unprotected Ether Withdrawal, SWC-106: Unprotected SELFDESTRUCT
    }

    fn recommendation(&self) -> &'static str {
        "Add access control modifiers (e.g., onlyOwner, onlyRole) to functions \
         that modify sensitive state or perform privileged operations."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-105",
            "https://swcregistry.io/docs/SWC-106",
        ]
    }
}
