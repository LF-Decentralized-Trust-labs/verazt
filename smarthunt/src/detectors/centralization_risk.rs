//! Centralization Risk detector.
//!
//! Detects centralization risks in contracts.

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::detectors::{Detector, ConfidenceLevel, create_bug};
use crate::detectors::AnalysisContext;
use solidity::ast::{ContractDef, ContractElem, Expr, FuncDef, Loc};
use solidity::ast::utils::Visit;

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
        let mut visitor = CentralizationRiskVisitor::new(self, context);
        for source_unit in &context.source_units {
            visitor.visit_source_unit(source_unit);
        }
        visitor.bugs
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

/// Visitor to collect centralization risk bugs.
struct CentralizationRiskVisitor<'a, 'b> {
    detector: &'a CentralizationRiskDetector,
    bugs: Vec<Bug>,
    privileged_functions: Vec<&'b solidity::ast::FuncDef>,
}

impl<'a, 'b> CentralizationRiskVisitor<'a, 'b> {
    fn new(detector: &'a CentralizationRiskDetector, _context: &'a AnalysisContext) -> Self {
        Self {
            detector,
            bugs: Vec::new(),
            privileged_functions: Vec::new(),
        }
    }
}

impl<'a, 'b> Visit<'b> for CentralizationRiskVisitor<'a, 'b> {
    fn visit_contract_def(&mut self, contract: &'b ContractDef) {
        // Reset for each contract
        self.privileged_functions.clear();

        // Collect privileged functions first
        for elem in &contract.body {
            if let ContractElem::Func(func) = elem {
                if self.detector.is_privileged_function(func) {
                    self.privileged_functions.push(func);
                }
            }
        }

        // Report if there are multiple privileged functions
        if self.privileged_functions.len() >= 3 {
            for func in &self.privileged_functions {
                let loc = func.loc.unwrap_or(Loc::new(1, 1, 1, 1));
                let bug = create_bug(
                    self.detector,
                    Some(&format!(
                        "Privileged function '{}' may pose centralization risk.",
                        func.name.base.as_str()
                    )),
                    loc,
                );
                self.bugs.push(bug);
            }
        }
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
