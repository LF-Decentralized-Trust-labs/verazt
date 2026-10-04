//! Missing Access Control Detector
//!
//! Detects public functions that modify state without access control guards.

use crate::detectors::sir::detector::{DetectionLevel, ScanDetector};
use crate::detectors::{ConfidenceLevel, DetectorId, DetectorMeta, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::ContractDecl;
use scirs::sir::dialect::EvmFunctionExt;
use scirs::sir::{MemberDecl, Module};

const META: DetectorMeta = DetectorMeta {
    bug_category: BugCategory::AccessControl,
    bug_kind: BugKind::Vulnerability,
    confidence: ConfidenceLevel::Medium,
    cwe_ids: &[284],
    description: "Detects public functions that modify state without access control",
    id: DetectorId::MissingAccessControl,
    name: "Missing Access Control",
    recommendation: "Add access control modifiers (e.g., `onlyOwner` or OpenZeppelin's \
         `AccessControl` with role-based checks) to functions that modify \
         sensitive state. Use `Ownable2Step` for ownership to prevent \
         accidental transfers.",
    references: &[
        "https://swcregistry.io/docs/SWC-105",
        "https://swcregistry.io/docs/SWC-106",
    ],
    risk_level: RiskLevel::High,
    swc_ids: &[105, 106],
    target: Target::Evm,
};

/// Scan detector for missing access control on public state-modifying functions.
#[derive(Debug, Default)]
pub struct MissingAccessControlDetector;

impl MissingAccessControlDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ScanDetector for MissingAccessControlDetector {
    fn meta(&self) -> &'static DetectorMeta {
        &META
    }

    fn level(&self) -> DetectionLevel {
        DetectionLevel::Contract
    }

    fn check_contract(&self, contract: &ContractDecl, _module: &Module) -> Vec<Bug> {
        let mut bugs = Vec::new();

        let storage_vars = contract.storage_names();
        if storage_vars.is_empty() {
            return bugs;
        }

        for member in &contract.members {
            if let MemberDecl::Function(func) = member {
                // Only check public/external functions.
                if !func.is_public() {
                    continue;
                }

                // Skip actual constructors / fallback / receive
                let is_ctor = func.attrs.iter().any(|a| {
                    a.namespace == "sir" && a.key == scirs::sir::evm_attrs::IS_CONSTRUCTOR
                });
                if is_ctor {
                    continue;
                }

                // Check for assert/require-based guard.
                let has_assert_guard = func.body.as_ref().map_or(false, |body| {
                    ContractDecl::has_assert_before_storage_write(body, &storage_vars)
                });

                if has_assert_guard {
                    continue;
                }

                // Check if function modifies state (structural walk)
                let has_writes_structural = func.body.as_ref().map_or(false, |body| {
                    ContractDecl::has_storage_write(body, &storage_vars)
                });

                if has_writes_structural {
                    bugs.push(META.bug(
                        Some(&format!(
                            "Function '{}' in '{}' performs state \
                             modifications without access control.",
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
    fn test_missing_access_control_detector() {
        let detector = MissingAccessControlDetector::new();
        assert_eq!(detector.meta().id, DetectorId::MissingAccessControl);
        assert_eq!(detector.meta().risk_level, RiskLevel::High);
    }
}
