//! Unchecked Call Return Detector (SIR structural)
//!
//! Detects low-level calls whose return values are not checked by walking
//! SIR function bodies.
//!
//! This detector finds:
//! - External calls used as expression statements (return value discarded)

use crate::context::AnalysisContext;
use crate::detectors::base::id::DetectorId;
use crate::detectors::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use crate::passes::base::Pass;
use crate::passes::base::meta::PassLevel;
use crate::passes::base::meta::PassRepresentation;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;
use scirs::sir::dialect::EvmCallExt;
use scirs::sir::utils::visit::{self, Visit};
use scirs::sir::{ContractDecl, ExprStmt, FunctionDecl};
use std::any::TypeId;

/// SIR structural detector for unchecked call return values.
#[derive(Debug, Default)]
pub struct UncheckedCallSirDetector;

impl UncheckedCallSirDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for UncheckedCallSirDetector {
    fn name(&self) -> &'static str {
        "Unchecked Call Return"
    }

    fn description(&self) -> &'static str {
        "Detects low-level calls whose return values are not checked using SIR tree walking"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Statement
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ir
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }
}

impl BugDetectionPass for UncheckedCallSirDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::UncheckedCall
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        if !context.has_ir() {
            return Ok(bugs);
        }

        struct Visitor<'b> {
            detector: &'b UncheckedCallSirDetector,
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

            fn visit_expr_stmt(&mut self, stmt: &'a ExprStmt) {
                if let scirs::sir::Expr::FunctionCall(call) = &stmt.expr {
                    if call.is_evm_external_call() {
                        self.bugs.push(Bug::new(
                            self.detector.name(),
                            Some(&format!(
                                "Unchecked call return value in '{}.{}'. \
                                 The return value of a low-level call is not checked.",
                                self.contract_name, self.func_name,
                            )),
                            stmt.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                            self.detector.bug_kind(),
                            self.detector.bug_category(),
                            self.detector.risk_level(),
                            self.detector.cwe_ids(),
                            self.detector.swc_ids(),
                            Some(self.detector.recommendation()),
                        ));
                    }
                }
                visit::default::visit_expr_stmt(self, stmt);
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
        BugCategory::UncheckedLowLevelCalls
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Medium
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::High
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![252] // CWE-252: Unchecked Return Value
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![104] // SWC-104: Unchecked Call Return Value
    }

    fn recommendation(&self) -> &'static str {
        "Ensure the return value of the low-level call is checked. \
         Use `require(success)` or handle the failure case explicitly."
    }

    fn references(&self) -> Vec<&'static str> {
        vec!["https://swcregistry.io/docs/SWC-104"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unchecked_call_detector() {
        let detector = UncheckedCallSirDetector::new();
        assert_eq!(detector.detector_id(), DetectorId::UncheckedCall);
        assert_eq!(detector.swc_ids(), vec![104]);
        assert_eq!(detector.risk_level(), RiskLevel::Medium);
    }
}
