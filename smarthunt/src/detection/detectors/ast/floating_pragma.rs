//! Floating Pragma Detector
//!
//! Detects unlocked compiler versions in pragma directives.

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::detection::pass::{BugDetectionPass, ConfidenceLevel, DetectorResult, create_bug};
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::analysis::context::AnalysisContext;
use solidity::ast::{PragmaKind, SourceUnitElem};

/// Detector for floating pragma.
pub struct FloatingPragmaDetector;

impl FloatingPragmaDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FloatingPragmaDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for FloatingPragmaDetector {
    fn id(&self) -> PassId {
        PassId::FloatingPragma
    }

    fn name(&self) -> &'static str {
        "Floating Pragma"
    }

    fn description(&self) -> &'static str {
        "Detects unlocked compiler versions that can lead to unexpected behavior."
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ast
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![]
    }
}

impl BugDetectionPass for FloatingPragmaDetector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        for source_unit in &context.source_units {
            for elem in &source_unit.elems {
                if let SourceUnitElem::Pragma(pragma) = elem {
                    if let PragmaKind::Version(version) = &pragma.kind {
                        // Check if version is floating (contains ^ or > or <)
                        if version.contains('^') || version.contains('>') || version.contains('<') {
                            if let Some(loc) = pragma.loc {
                                let bug = create_bug(
                                    self,
                                    Some(&format!("Floating pragma version: {}", version)),
                                    loc,
                                );
                                bugs.push(bug);
                            }
                        }
                    }
                }
            }
        }

        Ok(bugs)
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
        vec![103] // SWC-103: Floating Pragma
    }

    fn recommendation(&self) -> &'static str {
        "Lock the pragma version to a specific compiler version."
    }
}
