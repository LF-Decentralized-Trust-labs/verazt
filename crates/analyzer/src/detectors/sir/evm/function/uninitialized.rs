//! Uninitialized Storage Detector
//!
//! Detects uninitialized storage variables of mapping/array type.

use crate::detectors::sir::detector::{DetectionLevel, ScanDetector};
use crate::detectors::{ConfidenceLevel, DetectorId, DetectorMeta, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::dialect::EvmStorageExt;
use scirs::sir::{ContractDecl, MemberDecl, Module};

const META: DetectorMeta = DetectorMeta {
    bug_category: BugCategory::Other,
    bug_kind: BugKind::Vulnerability,
    confidence: ConfidenceLevel::Medium,
    cwe_ids: &[824],
    description: "Detects uninitialized storage variables using SIR tree walking",
    id: DetectorId::UninitializedStorage,
    name: "Uninitialized Storage",
    recommendation: "Initialize all storage variables explicitly. For local variables with storage \
         location, assign a reference to a state variable before use.",
    references: &["https://swcregistry.io/docs/SWC-109"],
    risk_level: RiskLevel::High,
    swc_ids: &[109],
    target: Target::Evm,
};

/// Scan detector for uninitialized storage variables.
#[derive(Debug, Default)]
pub struct UninitializedDetector;

impl UninitializedDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ScanDetector for UninitializedDetector {
    fn meta(&self) -> &'static DetectorMeta {
        &META
    }

    fn level(&self) -> DetectionLevel {
        DetectionLevel::Contract
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
                    bugs.push(META.bug(
                        Some(&format!(
                            "State variable '{}' in contract '{}' is not \
                             initialized. Consider initializing it explicitly.",
                            storage.name, contract.name,
                        )),
                        storage.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
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
        assert_eq!(detector.meta().id, DetectorId::UninitializedStorage);
        assert_eq!(detector.meta().risk_level, RiskLevel::High);
    }
}
