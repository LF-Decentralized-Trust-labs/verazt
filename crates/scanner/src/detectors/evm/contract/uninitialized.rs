//! Uninitialized Storage Detector
//!
//! Detects uninitialized storage variables of mapping/array type.

use crate::detector::{Confidence, DetectionLevel, ScanDetector, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::dialect::EvmStorageExt;
use scirs::sir::{ContractDecl, MemberDecl, Module};

/// Scan detector for uninitialized storage variables.
#[derive(Debug, Default)]
pub struct UninitializedDetector;

impl UninitializedDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ScanDetector for UninitializedDetector {
    fn id(&self) -> &'static str {
        "uninitialized-storage"
    }

    fn name(&self) -> &'static str {
        "Uninitialized Storage"
    }

    fn description(&self) -> &'static str {
        "Detects uninitialized storage variables using SIR tree walking"
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn bug_category(&self) -> BugCategory {
        BugCategory::Other
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::High
    }

    fn confidence(&self) -> Confidence {
        Confidence::Medium
    }

    fn target(&self) -> Target {
        Target::Evm
    }

    fn level(&self) -> DetectionLevel {
        DetectionLevel::Contract
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![824]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![109]
    }

    fn recommendation(&self) -> &'static str {
        "Initialize all storage variables explicitly. For local variables with storage \
         location, assign a reference to a state variable before use."
    }

    fn references(&self) -> Vec<&'static str> {
        vec!["https://swcregistry.io/docs/SWC-109"]
    }

    fn check_contract(&self, contract: &ContractDecl, _module: &Module) -> Vec<Bug> {
        let mut bugs = Vec::new();

        for member in &contract.members {
            if let MemberDecl::Storage(storage) = member {
                if storage.is_constant_storage() {
                    continue;
                }

                let ty_str = storage.ty.to_string().to_lowercase();
                let is_complex_type = ty_str.contains("mapping") || ty_str.contains("[]");

                if is_complex_type && storage.init.is_none() {
                    bugs.push(Bug::new(
                        self.name(),
                        Some(&format!(
                            "State variable '{}' in contract '{}' is not \
                             initialized. Consider initializing it explicitly.",
                            storage.name, contract.name,
                        )),
                        storage.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
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
    fn test_uninitialized_detector() {
        let detector = UninitializedDetector::new();
        assert_eq!(detector.id(), "uninitialized-storage");
        assert_eq!(detector.risk_level(), RiskLevel::High);
    }
}
