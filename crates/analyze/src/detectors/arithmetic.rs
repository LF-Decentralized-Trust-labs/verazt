//! AIR Arithmetic Detector
//!
//! Harvests findings from the `AIRArithmeticPass` analysis artifact.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::pipeline::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};

/// AIR-based arithmetic overflow detector.
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
    fn id(&self) -> PassId {
        PassId::AIRArithmetic
    }

    fn name(&self) -> &'static str {
        "AIR Arithmetic"
    }

    fn description(&self) -> &'static str {
        "Tainted-operand overflow detection via AIR dataflow"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Air
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::AIRArithmetic]
    }
}

impl BugDetectionPass for AIRArithmeticDetector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        // Findings are pre-computed by AIRArithmeticPass
        if let Some(findings) = context.get_artifact::<Vec<Bug>>("AIR.arithmetic_findings") {
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
