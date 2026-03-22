//! Variable Shadowing Detector (SIR structural)
//!
//! Detects variable shadowing across scopes by walking the SIR tree.
//! Flags local variable declarations that shadow storage variables or
//! parameters.

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

        for module in context.ir_units() {
            for decl in &module.decls {
                if let Decl::Contract(contract) = decl {
                    let state_vars: HashSet<String> =
                        structural::storage_names(contract).into_iter().collect();

                    if state_vars.is_empty() {
                        continue;
                    }

                    for member in &contract.members {
                        if let MemberDecl::Function(func) = member {
                            // Check parameters
                            for param in &func.params {
                                if state_vars.contains(&param.name) {
                                    bugs.push(Bug::new(
                                        self.name(),
                                        Some(&format!(
                                            "Parameter '{}' in '{}.{}' shadows a state variable.",
                                            param.name, contract.name, func.name,
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

                            // Check function body for local var declarations
                            if let Some(body) = &func.body {
                                structural::walk_stmts(body, &mut |stmt| {
                                    if let Stmt::LocalVar(lv) = stmt {
                                        for var in lv.vars.iter().flatten() {
                                            if state_vars.contains(&var.name) {
                                                bugs.push(Bug::new(
                                                    self.name(),
                                                    Some(&format!(
                                                        "Local variable '{}' in '{}.{}' shadows a state variable.",
                                                        var.name, contract.name, func.name,
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
