//! Floating Pragma Detector
//!
//! Detects unlocked compiler versions by inspecting the `#sir.pragma_solidity`
//! attribute on SIR modules.

use crate::detectors::sir::detector::{DetectionLevel, ScanDetector};
use crate::detectors::{ConfidenceLevel, DetectorId, DetectorMeta, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::AttrValue;
use scirs::sir::Module;
use scirs::sir::attrs::sir_attrs;

const META: DetectorMeta = DetectorMeta {
    bug_category: BugCategory::CodeQuality,
    bug_kind: BugKind::Refactoring,
    confidence: ConfidenceLevel::High,
    cwe_ids: &[],
    description: "Detects unlocked compiler versions from SIR module attrs.",
    id: DetectorId::FloatingPragma,
    name: "Floating Pragma",
    recommendation: "Lock the pragma to a specific compiler version (e.g., \
         `pragma solidity 0.8.20;` instead of `^0.8.20`). This ensures \
         the contract is tested and deployed with the same compiler version.",
    references: &["https://swcregistry.io/docs/SWC-103"],
    risk_level: RiskLevel::Low,
    swc_ids: &[103],
    target: Target::Evm,
};

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
    fn meta(&self) -> &'static DetectorMeta {
        &META
    }

    fn level(&self) -> DetectionLevel {
        DetectionLevel::Module
    }

    fn check_module(&self, module: &Module) -> Vec<Bug> {
        let mut bugs = Vec::new();

        for attr in &module.attrs {
            if attr.namespace == "sir" && attr.key == sir_attrs::PRAGMA_SOLIDITY {
                if let AttrValue::String(version) = &attr.value {
                    if Self::is_floating(version) {
                        let loc = attr.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0));
                        bugs.push(META.bug(
                            Some(&format!("Floating pragma version '{}'.", version)),
                            loc,
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
        assert_eq!(detector.meta().id, DetectorId::FloatingPragma);
        assert_eq!(detector.meta().swc_ids, &[103]);
        assert_eq!(detector.meta().risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_is_floating() {
        assert!(FloatingPragmaDetector::is_floating("^0.8.0"));
        assert!(FloatingPragmaDetector::is_floating(">=0.8.0"));
        assert!(FloatingPragmaDetector::is_floating(">=0.6.0 <0.9.0"));
        assert!(!FloatingPragmaDetector::is_floating("0.8.17"));
    }
}
