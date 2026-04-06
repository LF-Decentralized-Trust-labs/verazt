//! Visibility Detector
//!
//! Detects missing visibility specifiers on function declarations.

use crate::detector::{Confidence, DetectionLevel, ScanDetector, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::attrs::sir_attrs;
use scirs::sir::{ContractDecl, MemberDecl, Module};

/// Scan detector for missing function visibility specifiers.
#[derive(Debug, Default)]
pub struct VisibilityDetector;

impl VisibilityDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ScanDetector for VisibilityDetector {
    fn id(&self) -> &'static str {
        "visibility"
    }

    fn name(&self) -> &'static str {
        "Visibility Issues"
    }

    fn description(&self) -> &'static str {
        "Detects missing function visibility specifiers on SIR."
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn bug_category(&self) -> BugCategory {
        BugCategory::AccessControl
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Medium
    }

    fn confidence(&self) -> Confidence {
        Confidence::High
    }

    fn target(&self) -> Target {
        Target::Evm
    }

    fn level(&self) -> DetectionLevel {
        DetectionLevel::Contract
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![710]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![100, 108]
    }

    fn recommendation(&self) -> &'static str {
        "Explicitly set visibility (`public`, `external`, `internal`, or \
         `private`) for every function and state variable. In Solidity <0.5.0, \
         functions default to `public`, which may unintentionally expose \
         internal logic."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-100",
            "https://swcregistry.io/docs/SWC-108",
        ]
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
                    bugs.push(Bug::new(
                        self.name(),
                        Some(&format!(
                            "Function '{}' in contract '{}' has no explicit \
                             visibility specifier. Consider adding 'public', \
                             'external', 'internal', or 'private'.",
                            func.name, contract.name,
                        )),
                        func.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
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

        bugs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visibility_detector() {
        let detector = VisibilityDetector::new();
        assert_eq!(detector.id(), "visibility");
        assert_eq!(detector.risk_level(), RiskLevel::Medium);
    }
}
