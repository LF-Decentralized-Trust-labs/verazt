//! Centralization Risk Detector
//!
//! Detects centralization risks by identifying privileged functions that
//! have write sets covering security-sensitive storage variables.

use crate::detector::{Confidence, DetectionLevel, ScanDetector, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::{ContractDecl, MemberDecl, Module};

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
    fn id(&self) -> &'static str {
        "centralization-risk"
    }

    fn name(&self) -> &'static str {
        "Centralization Risk"
    }

    fn description(&self) -> &'static str {
        "Detects contracts with centralized control mechanisms"
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
        Confidence::Medium
    }

    fn target(&self) -> Target {
        Target::Evm
    }

    fn level(&self) -> DetectionLevel {
        DetectionLevel::Contract
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![250]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![]
    }

    fn recommendation(&self) -> &'static str {
        "Consider implementing timelocks, multi-sig requirements, or DAO-style \
         governance for privileged operations. Document the trust assumptions clearly."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://consensys.github.io/smart-contract-best-practices/development-recommendations/general/external-calls/",
        ]
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
                bugs.push(Bug::new(
                    self.name(),
                    Some(&format!(
                        "Privileged function '{}' in '{}' may pose \
                         centralization risk. Consider implementing \
                         timelocks or multi-sig for critical operations.",
                        fname, contract.name
                    )),
                    fspan.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                    self.bug_kind(),
                    self.bug_category(),
                    self.risk_level(),
                    self.cwe_ids(),
                    self.swc_ids(),
                    Some(self.recommendation()),
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
        assert_eq!(detector.id(), "centralization-risk");
        assert_eq!(detector.risk_level(), RiskLevel::Medium);
    }
}
