//! tx.origin Detector (SIR structural)
//!
//! Detects usage of `tx.origin` by walking the SIR tree for
//! `EvmExpr::TxOrigin` dialect nodes.

use crate::context::AnalysisContext;
use crate::detectors::base::id::DetectorId;
use crate::detectors::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use crate::passes::base::Pass;
use crate::passes::base::meta::PassLevel;
use crate::passes::base::meta::PassRepresentation;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;
use scirs::sir::dialect::evm::EvmExpr;
use scirs::sir::utils::visit::{self, Visit};
use scirs::sir::{ContractDecl, DialectExpr, FunctionDecl};
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

        struct Visitor<'b> {
            detector: &'b TxOriginSirDetector,
            bugs: &'b mut Vec<Bug>,
            contract_name: String,
            func_name: String,
        }

        impl<'a, 'b> Visit<'a> for Visitor<'b> {
            fn visit_contract_decl(&mut self, contract: &'a ContractDecl) {
                self.contract_name = contract.name.clone();
                visit::default::visit_contract_decl(self, contract);
            }

            fn visit_function_decl(&mut self, func: &'a FunctionDecl) {
                self.func_name = func.name.clone();
                visit::default::visit_function_decl(self, func);
            }

            fn visit_dialect_expr(&mut self, d: &'a DialectExpr) {
                if matches!(d, DialectExpr::Evm(EvmExpr::TxOrigin(_))) {
                    self.bugs.push(Bug::new(
                        self.detector.name(),
                        Some(&format!(
                            "tx.origin used in '{}.{}'. \
                             Consider using msg.sender instead.",
                            self.contract_name, self.func_name
                        )),
                        Loc::new(0, 0, 0, 0),
                        self.detector.bug_kind(),
                        self.detector.bug_category(),
                        self.detector.risk_level(),
                        self.detector.cwe_ids(),
                        self.detector.swc_ids(),
                    ));
                }
            }
        }

        let mut visitor = Visitor {
            detector: self,
            bugs: &mut bugs,
            contract_name: String::new(),
            func_name: String::new(),
        };
        visitor.visit_modules(context.ir_units());

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
