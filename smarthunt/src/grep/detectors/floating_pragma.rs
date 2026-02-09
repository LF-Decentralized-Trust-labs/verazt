//! Floating Pragma Detector (GREP-based)
//!
//! Detects unlocked compiler versions in pragma directives using
//! pattern matching.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::pipeline::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult, create_bug};
use bugs::bug::{Bug, BugKind, RiskLevel};
use solidity::ast::{PragmaKind, SourceUnitElem};

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

impl BugDetectionPass for FloatingPragmaGrepDetector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        for source_unit in &context.source_units {
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
        assert_eq!(detector.id(), PassId::FloatingPragma);
        assert_eq!(detector.swc_ids(), vec![103]);
        assert_eq!(detector.risk_level(), RiskLevel::Low);
    }
}
