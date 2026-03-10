//! Floating Pragma Detector (GREP-based)
//!
//! Detects unlocked compiler versions in pragma directives using
//! pattern matching.

use crate::detector::id::DetectorId;
use crate::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult, create_bug};
use analysis::context::AnalysisContext;
use analysis::pass::Pass;
use analysis::pass::meta::PassLevel;
use analysis::pass::meta::PassRepresentation;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::{PragmaKind, SourceUnit, SourceUnitElem};
use std::any::TypeId;

/// GREP-based detector for floating pragma.
///
/// Detects unlocked compiler versions that can lead to unexpected behavior.
#[derive(Debug, Default)]
pub struct FloatingPragmaGrepDetector;

impl FloatingPragmaGrepDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for FloatingPragmaGrepDetector {
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

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }
}

impl BugDetectionPass for FloatingPragmaGrepDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::FloatingPragma
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        let empty = vec![];
        let source_units: &Vec<SourceUnit> = context
            .get::<crate::artifacts::SourceUnitsArtifact>()
            .unwrap_or(&empty);

        for source_unit in source_units {
            for elem in &source_unit.elems {
                if let SourceUnitElem::Pragma(pragma) = elem {
                    if let PragmaKind::Version(version) = &pragma.kind {
                        // Check if version is floating (contains ^ or > or <)
                        if version.contains('^') || version.contains('>') || version.contains('<')
                        {
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

    fn bug_category(&self) -> BugCategory {
        BugCategory::CodeQuality
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

    fn references(&self) -> Vec<&'static str> {
        vec!["https://swcregistry.io/docs/SWC-103"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_floating_pragma_grep_detector() {
        let detector = FloatingPragmaGrepDetector::new();
        assert_eq!(detector.detector_id(), DetectorId::FloatingPragma);
        assert_eq!(detector.swc_ids(), vec![103]);
        assert_eq!(detector.risk_level(), RiskLevel::Low);
    }
}
