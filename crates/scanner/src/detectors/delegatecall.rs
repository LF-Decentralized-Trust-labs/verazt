//! Delegatecall Detector (SIR structural)
//!
//! Detects dangerous usage of delegatecall by walking the SIR tree for
//! `EvmExpr::Delegatecall` nodes or `.delegatecall()` field-access calls.

use crate::detector::id::DetectorId;
use crate::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use analysis::context::AnalysisContext;
use analysis::pass::Pass;
use analysis::pass::meta::PassLevel;
use analysis::pass::meta::PassRepresentation;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;
use scirs::sir::dialect::evm::EvmExpr;
use scirs::sir::utils::query as structural;
use scirs::sir::{Decl, DialectExpr, Expr, MemberDecl};
use std::any::TypeId;

/// SIR structural detector for delegatecall usage.
///
/// Delegatecall to untrusted addresses can lead to storage corruption
/// and complete contract compromise.
#[derive(Debug, Default)]
pub struct DelegatecallSirDetector;

impl DelegatecallSirDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for DelegatecallSirDetector {
    fn name(&self) -> &'static str {
        "Dangerous Delegatecall"
    }

    fn description(&self) -> &'static str {
        "Detects potentially dangerous delegatecall usage on SIR."
    }

    fn level(&self) -> PassLevel {
        PassLevel::Expression
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ir
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }
}

impl BugDetectionPass for DelegatecallSirDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::Delegatecall
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        if !context.has_ir() {
            return Ok(bugs);
        }

        for module in context.ir_units() {
            for decl in &module.decls {
                if let Decl::Contract(contract) = decl {
                    for member in &contract.members {
                        if let MemberDecl::Function(func) = member {
                            if let Some(body) = &func.body {
                                structural::walk_exprs(body, &mut |expr| {
                                    let is_delegatecall = match expr {
                                        // EVM dialect delegatecall node
                                        Expr::Dialect(DialectExpr::Evm(
                                            EvmExpr::Delegatecall { .. },
                                        )) => true,
                                        // Field-access fallback: addr.delegatecall(...)
                                        Expr::FieldAccess(fa) if fa.field == "delegatecall" => {
                                            true
                                        }
                                        _ => false,
                                    };
                                    if is_delegatecall {
                                        bugs.push(Bug::new(
                                            self.name(),
                                            Some(&format!(
                                                "Usage of delegatecall in '{}.{}'. \
                                                 Delegatecall to an untrusted address can lead \
                                                 to storage corruption and contract compromise.",
                                                contract.name, func.name
                                            )),
                                            Loc::new(0, 0, 0, 0),
                                            self.bug_kind(),
                                            self.bug_category(),
                                            self.risk_level(),
                                            self.cwe_ids(),
                                            self.swc_ids(),
                                        ));
                                    }
                                });
                            }
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
        RiskLevel::High
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::Medium
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![112] // SWC-112: Delegatecall to Untrusted Callee
    }

    fn recommendation(&self) -> &'static str {
        "Verify the target contract is trusted and update state variables carefully. \
         Consider using a library pattern instead of direct delegatecall."
    }

    fn references(&self) -> Vec<&'static str> {
        vec!["https://swcregistry.io/docs/SWC-112"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delegatecall_sir_detector() {
        let detector = DelegatecallSirDetector::new();
        assert_eq!(detector.detector_id(), DetectorId::Delegatecall);
        assert_eq!(detector.swc_ids(), vec![112]);
        assert_eq!(detector.risk_level(), RiskLevel::High);
    }
}
