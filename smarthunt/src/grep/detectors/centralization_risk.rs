//! Centralization Risk Detector (GREP-based)
//!
//! Detects centralization risks in smart contracts using pattern matching.
//! Finds contracts with privileged functions that give excessive control
//! to a single address or entity.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::pipeline::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult, create_bug};
use bugs::bug::{Bug, BugKind, RiskLevel};
use solidity::ast::{ContractDef, ContractElem, Expr, FuncDef, Loc, SourceUnitElem};

/// GREP-based detector for centralization risks.
#[derive(Debug, Default)]
pub struct CentralizationRiskGrepDetector;

impl CentralizationRiskGrepDetector {
    pub fn new() -> Self {
        Self
    }

    /// Risky function name patterns.
    const RISKY_FUNCTION_PATTERNS: &'static [&'static str] = &[
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

        let func_name = func.name.base.as_str().to_lowercase();
        Self::RISKY_FUNCTION_PATTERNS
            .iter()
            .any(|pattern| func_name.contains(pattern))
    }

    fn check_contract(&self, contract: &ContractDef, bugs: &mut Vec<Bug>) {
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
            for func in &privileged_functions {
                let loc = func.loc.unwrap_or(Loc::new(1, 1, 1, 1));
                let bug = create_bug(
                    self,
                    Some(&format!(
                        "Privileged function '{}' may pose centralization risk. \
                         Consider implementing timelocks or multi-sig for critical operations.",
                        func.name.base.as_str()
                    )),
                    loc,
                );
                bugs.push(bug);
            }
        }
    }
}

impl Pass for CentralizationRiskGrepDetector {
    fn id(&self) -> PassId {
        PassId::CentralizationRisk
    }

    fn name(&self) -> &'static str {
        "Centralization Risk"
    }

    fn description(&self) -> &'static str {
        "Detects contracts with centralized control mechanisms"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ast
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::SymbolTable, PassId::ModifierAnalysis]
    }
}

impl BugDetectionPass for CentralizationRiskGrepDetector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        for source_unit in &context.source_units {
            for elem in &source_unit.elems {
                if let SourceUnitElem::Contract(contract) = elem {
                    self.check_contract(contract, &mut bugs);
                }
            }
        }

        Ok(bugs)
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

    fn recommendation(&self) -> &'static str {
        "Consider implementing timelocks, multi-sig requirements, or DAO-style \
         governance for privileged operations. Document the trust assumptions clearly."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://consensys.github.io/smart-contract-best-practices/development-recommendations/general/external-calls/",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_centralization_risk_grep_detector() {
        let detector = CentralizationRiskGrepDetector::new();
        assert_eq!(detector.id(), PassId::CentralizationRisk);
        assert_eq!(detector.risk_level(), RiskLevel::Medium);
    }
}
