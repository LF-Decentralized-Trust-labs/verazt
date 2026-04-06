//! Unchecked Call Return Detector
//!
//! Detects low-level calls whose return values are not checked.

use crate::detector::{Confidence, DetectionLevel, ScanDetector, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::dialect::EvmCallExt;
use scirs::sir::utils::visit::{self, Visit};
use scirs::sir::{ContractDecl, ExprStmt, FunctionDecl, Module};

/// Scan detector for unchecked call return values.
#[derive(Debug, Default)]
pub struct UncheckedCallDetector;

impl UncheckedCallDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ScanDetector for UncheckedCallDetector {
    fn id(&self) -> &'static str {
        "unchecked-call"
    }

    fn name(&self) -> &'static str {
        "Unchecked Call Return"
    }

    fn description(&self) -> &'static str {
        "Detects low-level calls whose return values are not checked using SIR tree walking"
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

    fn confidence(&self) -> Confidence {
        Confidence::High
    }

    fn target(&self) -> Target {
        Target::Evm
    }

    fn level(&self) -> DetectionLevel {
        DetectionLevel::Function
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![252]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![104]
    }

    fn recommendation(&self) -> &'static str {
        "Ensure the return value of the low-level call is checked. \
         Use `require(success)` or handle the failure case explicitly."
    }

    fn references(&self) -> Vec<&'static str> {
        vec!["https://swcregistry.io/docs/SWC-104"]
    }

    fn check_function(
        &self,
        func: &FunctionDecl,
        contract: &ContractDecl,
        _module: &Module,
    ) -> Vec<Bug> {
        let mut bugs = Vec::new();

        struct Visitor<'b> {
            detector: &'b UncheckedCallDetector,
            bugs: &'b mut Vec<Bug>,
            contract_name: String,
            func_name: String,
        }

        impl<'a, 'b> Visit<'a> for Visitor<'b> {
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
            contract_name: contract.name.clone(),
            func_name: func.name.clone(),
        };
        visitor.visit_function_decl(func);

        bugs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unchecked_call_detector() {
        let detector = UncheckedCallDetector::new();
        assert_eq!(detector.id(), "unchecked-call");
        assert_eq!(detector.risk_level(), RiskLevel::Medium);
    }
}
