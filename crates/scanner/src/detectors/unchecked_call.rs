//! Unchecked Call Return Detector (SIR structural)
//!
//! Detects low-level calls whose return values are not checked by walking
//! SIR function bodies.
//!
//! This detector finds:
//! - External calls used as expression statements (return value discarded)

use crate::detector::id::DetectorId;
use crate::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use analysis::context::AnalysisContext;
use analysis::pass::Pass;
use analysis::pass::meta::PassLevel;
use analysis::pass::meta::PassRepresentation;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;
use scirs::sir::utils::query as structural;
use scirs::sir::{Decl, MemberDecl, Stmt};
use std::any::TypeId;

/// SIR structural detector for unchecked call return values.
#[derive(Debug, Default)]
pub struct UncheckedCallSirDetector;

impl UncheckedCallSirDetector {
    pub fn new() -> Self {
        Self
    }

    /// Walk statements looking for expression statements whose expression
    /// is an external call (return value discarded).
    fn check_stmts(
        &self,
        stmts: &[Stmt],
        contract_name: &str,
        func_name: &str,
        bugs: &mut Vec<Bug>,
    ) {
        for stmt in stmts {
            match stmt {
                // Expression statement — the return value is discarded.
                Stmt::Expr(expr_stmt) => {
                    if let scirs::sir::Expr::FunctionCall(call) = &expr_stmt.expr {
                        if structural::is_evm_external_call(call) {
                            bugs.push(Bug::new(
                                self.name(),
                                Some(&format!(
                                    "Unchecked call return value in '{}.{}'. \
                                     The return value of a low-level call is not checked.",
                                    contract_name, func_name,
                                )),
                                Loc::new(0, 0, 0, 0),
                                self.bug_kind(),
                                self.bug_category(),
                                self.risk_level(),
                                self.cwe_ids(),
                                self.swc_ids(),
                            ));
                        }
                    }
                }
                Stmt::If(s) => {
                    self.check_stmts(&s.then_body, contract_name, func_name, bugs);
                    if let Some(else_body) = &s.else_body {
                        self.check_stmts(else_body, contract_name, func_name, bugs);
                    }
                }
                Stmt::While(s) => {
                    self.check_stmts(&s.body, contract_name, func_name, bugs);
                }
                Stmt::For(s) => {
                    self.check_stmts(&s.body, contract_name, func_name, bugs);
                }
                Stmt::Block(inner) => {
                    self.check_stmts(inner, contract_name, func_name, bugs);
                }
                _ => {}
            }
        }
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

        for module in context.ir_units() {
            for decl in &module.decls {
                if let Decl::Contract(contract) = decl {
                    for member in &contract.members {
                        if let MemberDecl::Function(func) = member {
                            if let Some(body) = &func.body {
                                self.check_stmts(body, &contract.name, &func.name, &mut bugs);
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
