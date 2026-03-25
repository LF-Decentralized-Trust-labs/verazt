//! BIR Access Control Detector
//!
//! Harvests findings from the `AIRAccessControlPass` analysis artifact.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass::meta::PassLevel;
use crate::analysis::pass::meta::PassRepresentation;
use crate::detectors::id::DetectorId;
use crate::detectors::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use std::any::TypeId;

/// BIR-based access control detector.
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
    fn name(&self) -> &'static str {
        "BIR Access Control"
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

    fn dependencies(&self) -> Vec<TypeId> {
        // Note: The upstream AIRAccessControlPass was moved; the detection
        // logic now lives in this detector via artifact harvesting.
        vec![]
    }
}

impl BugDetectionPass for AIRAccessControlDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::AIRAccessControl
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        // Findings are pre-computed by AIRAccessControlPass
        #[allow(deprecated)]
        if let Some(findings) = context.get_artifact::<Vec<Bug>>("BIR.access_control_findings") {
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
