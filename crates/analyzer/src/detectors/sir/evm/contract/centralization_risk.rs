//! Centralization Risk Detector
//!
//! Detects centralization risks by identifying privileged functions that
//! have write sets covering security-sensitive storage variables.

use crate::detectors::sir::detector::{DetectionLevel, ScanDetector};
use crate::detectors::{ConfidenceLevel, DetectorId, DetectorMeta, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::{ContractDecl, MemberDecl, Module};

const META: DetectorMeta = DetectorMeta {
    bug_category: BugCategory::AccessControl,
    bug_kind: BugKind::Vulnerability,
    confidence: ConfidenceLevel::Medium,
    cwe_ids: &[250],
    description: "Detects contracts with centralized control mechanisms",
    id: DetectorId::CentralizationRisk,
    name: "Centralization Risk",
    recommendation: "Consider implementing timelocks, multi-sig requirements, or DAO-style \
         governance for privileged operations. Document the trust assumptions clearly.",
    references: &[
        "https://consensys.github.io/smart-contract-best-practices/development-recommendations/general/external-calls/",
    ],
    risk_level: RiskLevel::Medium,
    swc_ids: &[],
    target: Target::Evm,
};

/// Risky function name patterns indicating privileged operations.
const RISKY_FUNCTION_PATTERNS: &[&str] = &[
    "pause",
    "unpause",
    "freeze",
    "unfreeze",
    "setfee",
    "changefee",
    "updatefee",
    "setowner",
    "changeowner",
    "transferownership",
    "mint",
    "burn",
    "setprice",
    "changeprice",
    "setadmin",
    "addadmin",
    "removeadmin",
    "upgrade",
    "setimplementation",
    "emergencywithdraw",
    "drain",
    "blacklist",
    "whitelist",
];

/// Scan detector for centralization risks.
#[derive(Debug, Default)]
pub struct CentralizationRiskDetector;

impl CentralizationRiskDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ScanDetector for CentralizationRiskDetector {
    fn meta(&self) -> &'static DetectorMeta {
        &META
    }

    fn level(&self) -> DetectionLevel {
        DetectionLevel::Contract
    }

    fn check_contract(&self, contract: &ContractDecl, _module: &Module) -> Vec<Bug> {
        let mut bugs = Vec::new();

        let mut privileged_count = 0;
        let mut privileged_funcs: Vec<(String, Option<Loc>)> = Vec::new();

        for member in &contract.members {
            if let MemberDecl::Function(func) = member {
                // Check if function name matches risky patterns
                let func_lower = func.name.to_lowercase();
                let is_risky = RISKY_FUNCTION_PATTERNS
                    .iter()
                    .any(|p| func_lower.contains(p));

                if !is_risky {
                    continue;
                }

                // Check structurally for storage writes
                let has_structural_writes = func.body.as_ref().map_or(false, |body| {
                    let storage_vars = contract.storage_names();
                    ContractDecl::has_storage_write(body, &storage_vars)
                });

                if has_structural_writes {
                    privileged_count += 1;
                    privileged_funcs.push((func.name.clone(), func.span.clone()));
                }
            }
        }

        // Only report if there are multiple privileged functions
        if privileged_count >= 3 {
            for (fname, fspan) in &privileged_funcs {
                bugs.push(META.bug(
                    Some(&format!(
                        "Privileged function '{}' in '{}' may pose \
                         centralization risk. Consider implementing \
                         timelocks or multi-sig for critical operations.",
                        fname, contract.name
                    )),
                    fspan.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                ));
            }
        }

        bugs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_centralization_risk_detector() {
        let detector = CentralizationRiskDetector::new();
        assert_eq!(detector.meta().id, DetectorId::CentralizationRisk);
        assert_eq!(detector.meta().risk_level, RiskLevel::Medium);
    }
}
