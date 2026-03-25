//! BIR Arithmetic Detector
//!
//! Harvests findings from the `AIRArithmeticPass` analysis artifact.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass::meta::PassLevel;
use crate::analysis::pass::meta::PassRepresentation;
use crate::detectors::id::DetectorId;
use crate::detectors::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use std::any::TypeId;

/// BIR-based arithmetic overflow detector.
///
/// Harvests pre-computed findings from the `AIRArithmeticPass`.
#[derive(Debug, Default)]
pub struct AIRArithmeticDetector;

impl AIRArithmeticDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for AIRArithmeticDetector {
    fn name(&self) -> &'static str {
        "BIR Arithmetic"
    }

    fn description(&self) -> &'static str {
        "Tainted-operand overflow detection via BIR dataflow"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Air
    }

    fn dependencies(&self) -> Vec<TypeId> {
        // Note: The upstream AIRArithmeticPass was moved; the detection
        // logic now lives in this detector via artifact harvesting.
        vec![]
    }
}

impl BugDetectionPass for AIRArithmeticDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::AIRArithmetic
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        // Findings are pre-computed by AIRArithmeticPass
        #[allow(deprecated)]
        if let Some(findings) = context.get_artifact::<Vec<Bug>>("BIR.arithmetic_findings") {
            Ok(findings.clone())
        } else {
            Ok(vec![])
        }
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn bug_category(&self) -> BugCategory {
        BugCategory::Arithmetic
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::High
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::Medium
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![190]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![101]
    }

    fn recommendation(&self) -> &'static str {
        "Validate inputs and use checked math to prevent underflow or overflow"
    }
}
