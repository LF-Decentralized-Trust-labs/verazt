//! Floating Pragma Detector
//!
//! Detects unlocked compiler versions by inspecting the `#sir.pragma_solidity`
//! attribute on SIR modules.

use crate::detector::{Confidence, DetectionLevel, ScanDetector, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::AttrValue;
use scirs::sir::Module;
use scirs::sir::attrs::sir_attrs;

/// Scan detector for floating pragma.
#[derive(Debug, Default)]
pub struct FloatingPragmaDetector;

impl FloatingPragmaDetector {
    pub fn new() -> Self {
        Self
    }

    /// Returns true if the pragma version string is "floating" (non-pinned).
    fn is_floating(version: &str) -> bool {
        version.contains('^') || version.contains('>') || version.contains('<')
    }
}

impl ScanDetector for FloatingPragmaDetector {
    fn id(&self) -> &'static str {
        "floating-pragma"
    }

    fn name(&self) -> &'static str {
        "Floating Pragma"
    }

    fn description(&self) -> &'static str {
        "Detects unlocked compiler versions from SIR module attrs."
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

    fn confidence(&self) -> Confidence {
        Confidence::High
    }

    fn target(&self) -> Target {
        Target::Evm
    }

    fn level(&self) -> DetectionLevel {
        DetectionLevel::Module
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![103]
    }

    fn recommendation(&self) -> &'static str {
        "Lock the pragma to a specific compiler version (e.g., \
         `pragma solidity 0.8.20;` instead of `^0.8.20`). This ensures \
         the contract is tested and deployed with the same compiler version."
    }

    fn references(&self) -> Vec<&'static str> {
        vec!["https://swcregistry.io/docs/SWC-103"]
    }

    fn check_module(&self, module: &Module) -> Vec<Bug> {
        let mut bugs = Vec::new();

        for attr in &module.attrs {
            if attr.namespace == "sir" && attr.key == sir_attrs::PRAGMA_SOLIDITY {
                if let AttrValue::String(version) = &attr.value {
                    if Self::is_floating(version) {
                        let loc = attr.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0));
                        bugs.push(Bug::new(
                            self.name(),
                            Some(&format!("Floating pragma version '{}'.", version)),
                            loc,
                            self.bug_kind(),
                            self.bug_category(),
                            self.risk_level(),
                            self.cwe_ids(),
                            self.swc_ids(),
                            Some(self.recommendation()),
                        ));
                    }
                }
            }
        }

        bugs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_floating_pragma_detector() {
        let detector = FloatingPragmaDetector::new();
        assert_eq!(detector.id(), "floating-pragma");
        assert_eq!(detector.swc_ids(), vec![103]);
        assert_eq!(detector.risk_level(), RiskLevel::Low);
    }

    #[test]
    fn test_is_floating() {
        assert!(FloatingPragmaDetector::is_floating("^0.8.0"));
        assert!(FloatingPragmaDetector::is_floating(">=0.8.0"));
        assert!(FloatingPragmaDetector::is_floating(">=0.6.0 <0.9.0"));
        assert!(!FloatingPragmaDetector::is_floating("0.8.17"));
    }
}
