//! Visibility Detector
//!
//! Detects missing visibility specifiers on function declarations.

use crate::detectors::sir::detector::{DetectionLevel, ScanDetector};
use crate::detectors::{ConfidenceLevel, DetectorId, DetectorMeta, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::attrs::sir_attrs;
use scirs::sir::{ContractDecl, MemberDecl, Module};

const META: DetectorMeta = DetectorMeta {
    bug_category: BugCategory::AccessControl,
    bug_kind: BugKind::Vulnerability,
    confidence: ConfidenceLevel::High,
    cwe_ids: &[710],
    description: "Detects missing function visibility specifiers on SIR.",
    id: DetectorId::Visibility,
    name: "Visibility Issues",
    recommendation: "Explicitly set visibility (`public`, `external`, `internal`, or \
         `private`) for every function and state variable. In Solidity <0.5.0, \
         functions default to `public`, which may unintentionally expose \
         internal logic.",
    references: &[
        "https://swcregistry.io/docs/SWC-100",
        "https://swcregistry.io/docs/SWC-108",
    ],
    risk_level: RiskLevel::Medium,
    swc_ids: &[100, 108],
    target: Target::Evm,
};

/// Scan detector for missing function visibility specifiers.
#[derive(Debug, Default)]
pub struct VisibilityDetector;

impl VisibilityDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ScanDetector for VisibilityDetector {
    fn meta(&self) -> &'static DetectorMeta {
        &META
    }

    fn level(&self) -> DetectionLevel {
        DetectionLevel::Contract
    }

    fn check_contract(&self, contract: &ContractDecl, _module: &Module) -> Vec<Bug> {
        let mut bugs = Vec::new();

        for member in &contract.members {
            if let MemberDecl::Function(func) = member {
                if func.name.is_empty()
                    || func.name == "constructor"
                    || func.name == "fallback"
                    || func.name == "receive"
                {
                    continue;
                }

                let has_visibility = func
                    .attrs
                    .iter()
                    .any(|a| a.namespace == "sir" && a.key == sir_attrs::VISIBILITY);

                if !has_visibility {
                    bugs.push(META.bug(
                        Some(&format!(
                            "Function '{}' in contract '{}' has no explicit \
                             visibility specifier. Consider adding 'public', \
                             'external', 'internal', or 'private'.",
                            func.name, contract.name,
                        )),
                        func.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                    ));
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
    fn test_visibility_detector() {
        let detector = VisibilityDetector::new();
        assert_eq!(detector.meta().id, DetectorId::Visibility);
        assert_eq!(detector.meta().risk_level, RiskLevel::Medium);
    }
}
