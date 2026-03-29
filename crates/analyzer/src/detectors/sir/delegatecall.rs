//! Delegatecall Detector (SIR structural)
//!
//! Detects dangerous usage of delegatecall by walking the SIR tree for
//! `EvmExpr::Delegatecall` nodes or `.delegatecall()` field-access calls.

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
use scirs::sir::{ContractDecl, DialectExpr, FieldAccessExpr, FunctionDecl};
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

        struct Visitor<'b> {
            detector: &'b DelegatecallSirDetector,
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
                if let DialectExpr::Evm(EvmExpr::Delegatecall(e)) = d {
                    self.bugs.push(Bug::new(
                        self.detector.name(),
                        Some(&format!(
                            "Usage of delegatecall in '{}.{}'. \
                             Delegatecall to an untrusted address can lead \
                             to storage corruption and contract compromise.",
                            self.contract_name, self.func_name
                        )),
                        e.loc.clone(),
                        self.detector.bug_kind(),
                        self.detector.bug_category(),
                        self.detector.risk_level(),
                        self.detector.cwe_ids(),
                        self.detector.swc_ids(),
                    ));
                }
            }

            fn visit_field_access_expr(&mut self, fa: &'a FieldAccessExpr) {
                if fa.field == "delegatecall" {
                    self.bugs.push(Bug::new(
                        self.detector.name(),
                        Some(&format!(
                            "Usage of delegatecall in '{}.{}'. \
                             Delegatecall to an untrusted address can lead \
                             to storage corruption and contract compromise.",
                            self.contract_name, self.func_name
                        )),
                        fa.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                        self.detector.bug_kind(),
                        self.detector.bug_category(),
                        self.detector.risk_level(),
                        self.detector.cwe_ids(),
                        self.detector.swc_ids(),
                    ));
                }
                visit::default::visit_field_access_expr(self, fa);
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
