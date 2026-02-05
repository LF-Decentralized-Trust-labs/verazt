//! Centralization Risk detector.
//!
//! Detects centralization risks in contracts.

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::detectors::{Detector, ConfidenceLevel, create_bug};
use crate::engine::context::AnalysisContext;
use crate::passes::PassId;
use solidity::ast::{
    ContractDef, ContractElem, Expr, FuncDef, FuncVis, Loc, SourceUnit, SourceUnitElem,
};

/// Detector for centralization risks.
pub struct CentralizationRiskDetector;

impl CentralizationRiskDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CentralizationRiskDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for CentralizationRiskDetector {
    fn id(&self) -> &'static str {
        "centralization-risk"
    }

    fn name(&self) -> &'static str {
        "Centralization Risk"
    }

    fn description(&self) -> &'static str {
        "Contracts with privileged functions controlled by a single address \
         pose centralization risks. A compromised owner can drain funds or \
         manipulate the contract state."
    }

    fn required_passes(&self) -> Vec<PassId> {
        vec![PassId::SymbolTable, PassId::AccessControl]
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Medium
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::Medium
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![250] // CWE-250: Execution with Unnecessary Privileges
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![]
    }

    fn detect(&self, context: &AnalysisContext) -> Vec<Bug> {
        let mut bugs = Vec::new();

        for source_unit in &context.source_units {
            self.visit_source_unit(source_unit, context, &mut bugs);
        }

        bugs
    }

    fn recommendation(&self) -> &'static str {
        "Consider implementing timelocks, multi-sig requirements, or DAO-style \
         governance for privileged operations. Document the trust assumptions clearly."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://consensys.github.io/smart-contract-best-practices/development-recommendations/general/external-calls/#dont-rely-on-external-calls-for-input-validation",
        ]
    }
}

impl CentralizationRiskDetector {
    // Functions that indicate centralization risks
    const RISKY_FUNCTION_PATTERNS: &'static [&'static str] = &[
        "pause", "unpause", "freeze", "unfreeze",
        "setFee", "changeFee", "updateFee",
        "setOwner", "changeOwner", "transferOwnership",
        "mint", "burn",
        "setPrice", "changePrice",
        "setAdmin", "addAdmin", "removeAdmin",
        "upgrade", "setImplementation",
        "emergencyWithdraw", "drain",
        "blacklist", "whitelist",
    ];

    fn visit_source_unit(&self, source_unit: &SourceUnit, context: &AnalysisContext, bugs: &mut Vec<Bug>) {
        for elem in &source_unit.elems {
            if let SourceUnitElem::Contract(contract) = elem {
                self.visit_contract(contract, context, bugs);
            }
        }
    }

    fn visit_contract(&self, contract: &ContractDef, context: &AnalysisContext, bugs: &mut Vec<Bug>) {
        let mut privileged_functions = Vec::new();

        for elem in &contract.body {
            if let ContractElem::Func(func) = elem {
                if self.is_privileged_function(func) {
                    privileged_functions.push(func);
                }
            }
        }

        // Report if there are multiple privileged functions
        if privileged_functions.len() >= 3 {
            for func in privileged_functions {
                let loc = func.loc.unwrap_or(Loc::new(1, 1, 1, 1));
                let bug = create_bug(
                    self,
                    Some(&format!(
                        "Privileged function '{}' may pose centralization risk.",
                        func.name.base.as_str()
                    )),
                    loc,
                );
                bugs.push(bug);
            }
        }
    }

    fn is_privileged_function(&self, func: &FuncDef) -> bool {
        // Check if function has owner/admin modifier
        let has_privilege_modifier = func.modifier_invocs.iter().any(|m| {
            if let Expr::Ident(ident) = m.callee.as_ref() {
                let name = ident.name.base.as_str().to_lowercase();
                name.contains("owner") || name.contains("admin") || name.contains("role")
            } else {
                false
            }
        });

        if !has_privilege_modifier {
            return false;
        }

        // Check if function name matches risky patterns
        let func_name = func.name.base.as_str().to_lowercase();
        Self::RISKY_FUNCTION_PATTERNS.iter()
            .any(|pattern| func_name.contains(&pattern.to_lowercase()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_centralization_risk_detector_new() {
        let detector = CentralizationRiskDetector::new();
        assert_eq!(detector.id(), "centralization-risk");
    }
}
