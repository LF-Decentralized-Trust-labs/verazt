//! AIR Access Control Detector
//!
//! Harvests findings from the `AIRAccessControlPass` analysis artifact.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::pipeline::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};

/// AIR-based access control detector.
///
/// Harvests pre-computed findings from the `AIRAccessControlPass`.
#[derive(Debug, Default)]
pub struct AIRAccessControlDetector;

impl AIRAccessControlDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for AIRAccessControlDetector {
    fn id(&self) -> PassId {
        PassId::AIRAccessControl
    }

    fn name(&self) -> &'static str {
        "AIR Access Control"
    }

    fn description(&self) -> &'static str {
        "Access control: taint-flow path to storage write lacks SignerArg guard"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Air
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::AIRAccessControl]
    }
}

impl BugDetectionPass for AIRAccessControlDetector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        // Findings are pre-computed by AIRAccessControlPass
        if let Some(findings) = context.get_artifact::<Vec<Bug>>("AIR.access_control_findings") {
            Ok(findings.clone())
        } else {
            Ok(vec![])
        }
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn bug_category(&self) -> BugCategory {
        BugCategory::AccessControl
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::High
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::Medium
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![284]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![105]
    }

    fn recommendation(&self) -> &'static str {
        "Ensure access controls properly authenticate the caller before state changes"
    }
}
