//! Centralization Risk Detector (SIR structural + WriteSetPass)
//!
//! Detects centralization risks by identifying privileged functions that
//! have write sets covering security-sensitive storage variables.

use crate::context::AnalysisContext;
use crate::detectors::base::id::DetectorId;
use crate::detectors::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use crate::passes::base::Pass;
use crate::passes::base::meta::PassLevel;
use crate::passes::base::meta::PassRepresentation;
use crate::passes::sir::WriteSetArtifact;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;

use scirs::sir::{ContractDecl, Decl, MemberDecl};
use std::any::TypeId;

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

/// SIR structural detector for centralization risks.
#[derive(Debug, Default)]
pub struct CentralizationRiskSirDetector;

impl CentralizationRiskSirDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for CentralizationRiskSirDetector {
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
        PassRepresentation::Ir
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![TypeId::of::<crate::passes::sir::WriteSetPass>()]
    }
}

impl BugDetectionPass for CentralizationRiskSirDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::CentralizationRisk
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        if !context.has_ir() {
            return Ok(bugs);
        }

        let write_sets = context.get::<WriteSetArtifact>();

        for module in context.ir_units() {
            for decl in &module.decls {
                if let Decl::Contract(contract) = decl {
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

                            // Only flag if function also has a non-empty write set
                            let has_writes = write_sets
                                .and_then(|ws| ws.get(&(contract.name.clone(), func.name.clone())))
                                .map_or(false, |w| !w.is_empty());

                            // Or check structurally
                            let has_structural_writes = func.body.as_ref().map_or(false, |body| {
                                let storage_vars = contract.storage_names();
                                ContractDecl::has_storage_write(body, &storage_vars)
                            });

                            if has_writes || has_structural_writes {
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
                            ));
                        }
                    }
                }
            }
        }

        Ok(bugs)
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
    fn test_centralization_risk_sir_detector() {
        let detector = CentralizationRiskSirDetector::new();
        assert_eq!(detector.detector_id(), DetectorId::CentralizationRisk);
        assert_eq!(detector.risk_level(), RiskLevel::Medium);
    }
}
