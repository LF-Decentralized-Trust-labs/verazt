//! Floating Pragma Detector (SIR structural)
//!
//! Detects unlocked compiler versions by inspecting the `#sir.pragma_solidity`
//! attribute on SIR modules.

use crate::detector::id::DetectorId;
use crate::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use analysis::context::AnalysisContext;
use analysis::pass::Pass;
use analysis::pass::meta::PassLevel;
use analysis::pass::meta::PassRepresentation;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;
use mlir::sir::AttrValue;
use mlir::sir::attrs::sir_attrs;
use std::any::TypeId;

/// SIR structural detector for floating pragma.
///
/// Inspects `module.attrs` for `#sir.pragma_solidity`. If the version uses
/// `^` or open-ended `>=` without an upper bound, flags it.
#[derive(Debug, Default)]
pub struct FloatingPragmaSirDetector;

impl FloatingPragmaSirDetector {
    pub fn new() -> Self {
        Self
    }

    /// Returns true if the pragma version string is "floating" (non-pinned).
    fn is_floating(version: &str) -> bool {
        version.contains('^') || version.contains('>') || version.contains('<')
    }
}

impl Pass for FloatingPragmaSirDetector {
    fn name(&self) -> &'static str {
        "Floating Pragma"
    }

    fn description(&self) -> &'static str {
        "Detects unlocked compiler versions from SIR module attrs."
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ir
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }
}

impl BugDetectionPass for FloatingPragmaSirDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::FloatingPragma
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        if !context.has_ir() {
            return Ok(bugs);
        }

        for module in context.ir_units() {
            for attr in &module.attrs {
                if attr.namespace == "sir" && attr.key == sir_attrs::PRAGMA_SOLIDITY {
                    if let AttrValue::String(version) = &attr.value {
                        if Self::is_floating(version) {
                            bugs.push(Bug::new(
                                self.name(),
                                Some(&format!(
                                    "Floating pragma version '{}' in module '{}'.",
                                    version, module.id
                                )),
                                Loc::new(0, 0, 0, 0),
                                self.bug_kind(),
                                self.bug_category(),
                                self.risk_level(),
                                self.cwe_ids(),
                                self.swc_ids(),
                            ));
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
    fn test_floating_pragma_sir_detector() {
        let detector = FloatingPragmaSirDetector::new();
        assert_eq!(detector.detector_id(), DetectorId::FloatingPragma);
        assert_eq!(detector.swc_ids(), vec![103]);
        assert_eq!(detector.risk_level(), RiskLevel::Low);
    }

    #[test]
    fn test_is_floating() {
        assert!(FloatingPragmaSirDetector::is_floating("^0.8.0"));
        assert!(FloatingPragmaSirDetector::is_floating(">=0.8.0"));
        assert!(FloatingPragmaSirDetector::is_floating(">=0.6.0 <0.9.0"));
        assert!(!FloatingPragmaSirDetector::is_floating("0.8.17"));
    }
}
