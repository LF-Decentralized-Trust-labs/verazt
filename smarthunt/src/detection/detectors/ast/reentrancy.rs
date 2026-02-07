//! Reentrancy Detector (Stub)
//!
//! Detects potential reentrancy vulnerabilities.
//!
//! NOTE: This is currently a stub implementation. Full reentrancy detection
//! requires call graph analysis and state mutation tracking, which will be
//! implemented once the CFG and call graph passes are available.

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::detection::pass::{BugDetectionPass, ConfidenceLevel, DetectorResult};

/// Detector for reentrancy vulnerabilities.
#[derive(Debug, Default)]
pub struct ReentrancyDetector;

impl ReentrancyDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for ReentrancyDetector {
    fn id(&self) -> PassId {
        PassId::Reentrancy
    }

    fn name(&self) -> &'static str {
        "Reentrancy"
    }

    fn description(&self) -> &'static str {
        "Detects potential reentrancy vulnerabilities (requires CFG support)"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ast
    }

    fn dependencies(&self) -> Vec<PassId> {
        // Will require: CFG, CallGraph, StateAccess
        vec![]
    }
}

impl BugDetectionPass for ReentrancyDetector {
    fn detect(&self, _context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        // TODO: Implement full reentrancy detection
        // This requires:
        // 1. Call graph to track external calls
        // 2. State mutation analysis to track state changes
        // 3. CFG to determine ordering of calls and state changes
        Ok(vec![])
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Critical
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::Medium
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![841] // CWE-841: Improper Enforcement of Behavioral Workflow
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![107] // SWC-107: Reentrancy
    }

    fn recommendation(&self) -> &'static str {
        "Follow the Checks-Effects-Interactions pattern: perform all state changes \
         before making external calls. Consider using a reentrancy guard (e.g., OpenZeppelin's ReentrancyGuard)."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-107",
            "https://consensys.github.io/smart-contract-best-practices/attacks/reentrancy/",
        ]
    }
}
