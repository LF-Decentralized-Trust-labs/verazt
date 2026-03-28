//! Variable Shadowing Detector (SIR structural)
//!
//! Detects variable shadowing across scopes by walking the SIR tree.
//! Flags local variable declarations that shadow storage variables or
//! parameters.

use crate::context::AnalysisContext;
use crate::detectors::base::id::DetectorId;
use crate::detectors::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use crate::passes::base::Pass;
use crate::passes::base::meta::PassLevel;
use crate::passes::base::meta::PassRepresentation;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;

use scirs::sir::utils::visit::{self, Visit};
use scirs::sir::{ContractDecl, FunctionDecl, LocalVarStmt};
use std::any::TypeId;
use std::collections::HashSet;

/// SIR structural detector for variable shadowing.
///
/// Detects local variables that shadow state variables in the same contract.
#[derive(Debug, Default)]
pub struct ShadowingSirDetector;

impl ShadowingSirDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for ShadowingSirDetector {
    fn name(&self) -> &'static str {
        "Variable Shadowing"
    }

    fn description(&self) -> &'static str {
        "Detects variable shadowing that can cause confusion."
    }

    fn level(&self) -> PassLevel {
        PassLevel::Variable
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ir
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }
}

impl BugDetectionPass for ShadowingSirDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::Shadowing
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        if !context.has_ir() {
            return Ok(bugs);
        }

        struct Visitor<'b> {
            detector: &'b ShadowingSirDetector,
            bugs: &'b mut Vec<Bug>,
            contract_name: String,
            func_name: String,
            state_vars: HashSet<String>,
        }

        impl<'a, 'b> Visit<'a> for Visitor<'b> {
            fn visit_contract_decl(&mut self, contract: &'a ContractDecl) {
                self.contract_name = contract.name.clone();
                self.state_vars = contract.storage_names().into_iter().collect();
                if !self.state_vars.is_empty() {
                    visit::default::visit_contract_decl(self, contract);
                }
            }

            fn visit_function_decl(&mut self, func: &'a FunctionDecl) {
                self.func_name = func.name.clone();
                // Check parameters for shadowing
                for param in &func.params {
                    if self.state_vars.contains(&param.name) {
                        self.bugs.push(Bug::new(
                            self.detector.name(),
                            Some(&format!(
                                "Parameter '{}' in '{}.{}' shadows a state variable.",
                                param.name, self.contract_name, self.func_name,
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
                visit::default::visit_function_decl(self, func);
            }

            fn visit_local_var_stmt(&mut self, stmt: &'a LocalVarStmt) {
                for var in stmt.vars.iter().flatten() {
                    if self.state_vars.contains(&var.name) {
                        self.bugs.push(Bug::new(
                            self.detector.name(),
                            Some(&format!(
                                "Local variable '{}' in '{}.{}' shadows a state variable.",
                                var.name, self.contract_name, self.func_name,
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
                visit::default::visit_local_var_stmt(self, stmt);
            }
        }

        let mut visitor = Visitor {
            detector: self,
            bugs: &mut bugs,
            contract_name: String::new(),
            func_name: String::new(),
            state_vars: HashSet::new(),
        };
        visitor.visit_modules(context.ir_units());

        Ok(bugs)
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

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::High
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![119] // SWC-119: Shadowing State Variables
    }

    fn recommendation(&self) -> &'static str {
        "Rename the shadowed variable to remove ambiguity."
    }

    fn references(&self) -> Vec<&'static str> {
        vec!["https://swcregistry.io/docs/SWC-119"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shadowing_sir_detector() {
        let detector = ShadowingSirDetector::new();
        assert_eq!(detector.detector_id(), DetectorId::Shadowing);
        assert_eq!(detector.swc_ids(), vec![119]);
    }
}
