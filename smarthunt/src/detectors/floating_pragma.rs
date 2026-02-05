//! Floating pragma detector.
//!
//! Detects floating pragma versions.

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::detectors::{Detector, ConfidenceLevel, create_bug};
use crate::engine::context::AnalysisContext;
use crate::passes::PassId;
use solidity::ast::{Loc, PragmaKind, SourceUnit, SourceUnitElem};

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

impl Detector for FloatingPragmaDetector {
    fn id(&self) -> &'static str {
        "floating-pragma"
    }

    fn name(&self) -> &'static str {
        "Floating Pragma"
    }

    fn description(&self) -> &'static str {
        "Using a floating pragma (e.g., ^0.8.0) allows the contract to be compiled \
         with any compatible compiler version, which may introduce bugs or \
         inconsistent behavior across deployments."
    }

    fn required_passes(&self) -> Vec<PassId> {
        vec![]
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

    fn detect(&self, context: &AnalysisContext) -> Vec<Bug> {
        let mut bugs = Vec::new();
        
        for source_unit in &context.source_units {
            self.check_source_unit(source_unit, &mut bugs);
        }
        
        bugs
    }

    fn recommendation(&self) -> &'static str {
        "Lock the pragma to a specific compiler version (e.g., pragma solidity 0.8.20;)."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-103",
        ]
    }
}

impl FloatingPragmaDetector {
    fn check_source_unit(&self, source_unit: &SourceUnit, bugs: &mut Vec<Bug>) {
        for elem in &source_unit.elems {
            if let SourceUnitElem::Pragma(pragma) = elem {
                if let PragmaKind::Version(version) = &pragma.kind {
                    if self.is_floating(version) {
                        let loc = pragma.loc.unwrap_or(Loc::new(1, 1, 1, 1));
                        let bug = create_bug(
                            self,
                            Some(&format!("Floating pragma detected: {}", version)),
                            loc,
                        );
                        bugs.push(bug);
                    }
                }
            }
        }
    }

    fn is_floating(&self, version: &str) -> bool {
        // Check for floating version indicators
        version.contains('^') 
            || version.contains('>')
            || version.contains('<')
            || version.contains(">=")
            || version.contains("<=")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_floating_pragma_detector() {
        let detector = FloatingPragmaDetector::new();
        assert_eq!(detector.id(), "floating-pragma");
        assert!(detector.is_floating("^0.8.0"));
        assert!(detector.is_floating(">=0.8.0"));
        assert!(!detector.is_floating("0.8.20"));
    }
}
