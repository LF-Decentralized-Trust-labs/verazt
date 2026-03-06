//! ANIR Access Control Detector
//!
//! Harvests findings from the `AnirAccessControlPass` analysis artifact.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::pipeline::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};

/// ANIR-based access control detector.
///
/// Harvests pre-computed findings from the `AnirAccessControlPass`.
#[derive(Debug, Default)]
pub struct AnirAccessControlDetector;

impl AnirAccessControlDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for AnirAccessControlDetector {
    fn id(&self) -> PassId {
        PassId::AnirAccessControl
    }

    fn name(&self) -> &'static str {
        "ANIR Access Control"
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
        vec![PassId::AnirAccessControl]
    }
}

impl BugDetectionPass for AnirAccessControlDetector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        // Findings are pre-computed by AnirAccessControlPass
        if let Some(findings) = context.get_artifact::<Vec<Bug>>("anir.access_control_findings") {
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
}
