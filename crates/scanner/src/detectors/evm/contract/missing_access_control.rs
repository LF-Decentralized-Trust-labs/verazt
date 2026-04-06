//! Missing Access Control Detector
//!
//! Detects public functions that modify state without access control guards.

use crate::detector::{Confidence, DetectionLevel, ScanDetector, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::ContractDecl;
use scirs::sir::dialect::EvmFunctionExt;
use scirs::sir::{MemberDecl, Module};

/// Scan detector for missing access control on public state-modifying functions.
#[derive(Debug, Default)]
pub struct MissingAccessControlDetector;

impl MissingAccessControlDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ScanDetector for MissingAccessControlDetector {
    fn id(&self) -> &'static str {
        "missing-access-control"
    }

    fn name(&self) -> &'static str {
        "Missing Access Control"
    }

    fn description(&self) -> &'static str {
        "Detects public functions that modify state without access control"
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn bug_category(&self) -> BugCategory {
        BugCategory::AccessControl
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
        vec![284]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![105, 106]
    }

    fn recommendation(&self) -> &'static str {
        "Add access control modifiers (e.g., `onlyOwner` or OpenZeppelin's \
         `AccessControl` with role-based checks) to functions that modify \
         sensitive state. Use `Ownable2Step` for ownership to prevent \
         accidental transfers."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-105",
            "https://swcregistry.io/docs/SWC-106",
        ]
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
                    bugs.push(Bug::new(
                        self.name(),
                        Some(&format!(
                            "Function '{}' in '{}' performs state \
                             modifications without access control.",
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
    fn test_missing_access_control_detector() {
        let detector = MissingAccessControlDetector::new();
        assert_eq!(detector.id(), "missing-access-control");
        assert_eq!(detector.risk_level(), RiskLevel::High);
    }
}
