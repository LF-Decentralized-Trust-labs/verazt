//! Variable Shadowing Detector
//!
//! Detects local variable declarations that shadow storage variables.

use crate::detector::{Confidence, DetectionLevel, ScanDetector, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::utils::visit::{self, Visit};
use scirs::sir::{ContractDecl, FunctionDecl, LocalVarStmt, Module};
use std::collections::HashSet;

/// Scan detector for variable shadowing.
#[derive(Debug, Default)]
pub struct ShadowingDetector;

impl ShadowingDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ScanDetector for ShadowingDetector {
    fn id(&self) -> &'static str {
        "shadowing"
    }

    fn name(&self) -> &'static str {
        "Variable Shadowing"
    }

    fn description(&self) -> &'static str {
        "Detects variable shadowing that can cause confusion."
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Refactoring
    }

    fn bug_category(&self) -> BugCategory {
        BugCategory::CodeQuality
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Low
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
        vec![]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![119]
    }

    fn recommendation(&self) -> &'static str {
        "Rename the local variable to avoid shadowing the inherited state \
         variable. Shadowing can cause unintended reads/writes to the wrong \
         variable, leading to subtle logic bugs."
    }

    fn references(&self) -> Vec<&'static str> {
        vec!["https://swcregistry.io/docs/SWC-119"]
    }

    fn check_function(
        &self,
        func: &FunctionDecl,
        contract: &ContractDecl,
        _module: &Module,
    ) -> Vec<Bug> {
        let mut bugs = Vec::new();

        let state_vars: HashSet<String> = contract.storage_names().into_iter().collect();
        if state_vars.is_empty() {
            return bugs;
        }

        // Check parameters for shadowing
        for param in &func.params {
            if state_vars.contains(&param.name) {
                bugs.push(Bug::new(
                    self.name(),
                    Some(&format!(
                        "Parameter '{}' in '{}.{}' shadows a state variable.",
                        param.name, contract.name, func.name,
                    )),
                    func.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                    self.bug_kind(),
                    self.bug_category(),
                    self.risk_level(),
                    self.cwe_ids(),
                    self.swc_ids(),
                    Some(self.recommendation()),
                ));
            }
        }

        // Check local variable declarations
        struct Visitor<'b> {
            detector: &'b ShadowingDetector,
            bugs: &'b mut Vec<Bug>,
            contract_name: String,
            func_name: String,
            state_vars: HashSet<String>,
        }

        impl<'a, 'b> Visit<'a> for Visitor<'b> {
            fn visit_local_var_stmt(&mut self, stmt: &'a LocalVarStmt) {
                for var in stmt.vars.iter().flatten() {
                    if self.state_vars.contains(&var.name) {
                        self.bugs.push(Bug::new(
                            self.detector.name(),
                            Some(&format!(
                                "Local variable '{}' in '{}.{}' shadows a state variable.",
                                var.name, self.contract_name, self.func_name,
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
                visit::default::visit_local_var_stmt(self, stmt);
            }
        }

        let mut visitor = Visitor {
            detector: self,
            bugs: &mut bugs,
            contract_name: contract.name.clone(),
            func_name: func.name.clone(),
            state_vars,
        };
        visitor.visit_function_decl(func);

        bugs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shadowing_detector() {
        let detector = ShadowingDetector::new();
        assert_eq!(detector.id(), "shadowing");
        assert_eq!(detector.risk_level(), RiskLevel::Low);
    }
}
