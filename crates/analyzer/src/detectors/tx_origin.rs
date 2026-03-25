//! tx.origin Detector (SIR structural)
//!
//! Detects usage of `tx.origin` by walking the SIR tree for
//! `EvmExpr::TxOrigin` dialect nodes.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass::meta::PassLevel;
use crate::analysis::pass::meta::PassRepresentation;
use crate::detectors::id::DetectorId;
use crate::detectors::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;
use scirs::sir::dialect::evm::EvmExpr;
use scirs::sir::utils::query as structural;
use scirs::sir::{Decl, DialectExpr, Expr, MemberDecl};
use std::any::TypeId;

/// SIR structural detector for tx.origin usage.
///
/// Using tx.origin for authentication is vulnerable to phishing attacks.
#[derive(Debug, Default)]
pub struct TxOriginSirDetector;

impl TxOriginSirDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for TxOriginSirDetector {
    fn name(&self) -> &'static str {
        "Dangerous use of tx.origin"
    }

    fn description(&self) -> &'static str {
        "Using tx.origin for authentication is vulnerable to phishing attacks."
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

impl BugDetectionPass for TxOriginSirDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::TxOrigin
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
                                    if matches!(
                                        expr,
                                        Expr::Dialect(DialectExpr::Evm(EvmExpr::TxOrigin))
                                    ) {
                                        bugs.push(Bug::new(
                                            self.name(),
                                            Some(&format!(
                                                "tx.origin used in '{}.{}'. \
                                                 Consider using msg.sender instead.",
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
        ConfidenceLevel::High
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![345] // CWE-345: Insufficient Verification of Data Authenticity
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![115] // SWC-115: Authorization through tx.origin
    }

    fn recommendation(&self) -> &'static str {
        "Use msg.sender instead of tx.origin for authentication."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-115",
            "https://consensys.github.io/smart-contract-best-practices/development-recommendations/solidity-specific/tx-origin/",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx_origin_sir_detector() {
        let detector = TxOriginSirDetector::new();
        assert_eq!(detector.detector_id(), DetectorId::TxOrigin);
        assert_eq!(detector.swc_ids(), vec![115]);
        assert_eq!(detector.risk_level(), RiskLevel::High);
    }
}
