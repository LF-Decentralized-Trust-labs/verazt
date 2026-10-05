//! Constant State Variable Detector
//!
//! Detects state variables that could be declared constant or immutable
//! by checking if they are initialized but never modified.

use crate::detectors::sir::detector::{DetectionLevel, ScanDetector};
use crate::detectors::{ConfidenceLevel, DetectorId, DetectorMeta, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::dialect::EvmStorageExt;
use scirs::sir::{ContractDecl, MemberDecl, Module};

const META: DetectorMeta = DetectorMeta {
    bug_category: BugCategory::CodeQuality,
    bug_kind: BugKind::Optimization,
    confidence: ConfidenceLevel::High,
    cwe_ids: &[],
    description: "Detects state variables that could be constant/immutable",
    id: DetectorId::ConstantStateVar,
    name: "Constant State Variable",
    recommendation: "Use 'constant' for compile-time constants and 'immutable' for values \
         set once in the constructor. This saves gas by not using storage.",
    references: &[
        "https://docs.soliditylang.org/en/latest/contracts.html#constant-and-immutable-state-variables",
    ],
    risk_level: RiskLevel::Low,
    swc_ids: &[],
    target: Target::Evm,
};

/// Scan detector for state variables that could be constant.
#[derive(Debug, Default)]
pub struct ConstantStateVarDetector;

impl ConstantStateVarDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ScanDetector for ConstantStateVarDetector {
    fn meta(&self) -> &'static DetectorMeta {
        &META
    }

    fn level(&self) -> DetectionLevel {
        DetectionLevel::Contract
    }

    fn check_contract(&self, contract: &ContractDecl, _module: &Module) -> Vec<Bug> {
        let mut bugs = Vec::new();

        // Collect all written storage vars across all functions (structural
        // check)
        let storage_vars = contract.storage_names();
        let mut all_written = std::collections::HashSet::new();
        for member in &contract.members {
            if let MemberDecl::Function(func) = member {
                if let Some(body) = &func.body {
                    for sv in &storage_vars {
                        if ContractDecl::has_storage_write(body, &[sv.clone()]) {
                            all_written.insert(sv.clone());
                        }
                    }
                }
            }
        }

        for member in &contract.members {
            if let MemberDecl::Storage(storage) = member {
                // Skip already constant/immutable
                if storage.is_constant_storage() {
                    continue;
                }

                // Only flag variables with an initializer
                if storage.init.is_none() {
                    continue;
                }

                // If not in any function's write set → effectively constant
                if !all_written.contains(&storage.name) {
                    bugs.push(META.bug(
                        Some(&format!(
                            "State variable '{}' in '{}' is never modified after \
                             initialization. Consider declaring it as 'constant' \
                             or 'immutable' to save gas.",
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
    fn test_constant_state_var_detector() {
        let detector = ConstantStateVarDetector::new();
        assert_eq!(detector.meta().id, DetectorId::ConstantStateVar);
        assert_eq!(detector.meta().risk_level, RiskLevel::Low);
    }
}
