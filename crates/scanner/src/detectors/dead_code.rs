//! Dead Code Detector (SIR structural)
//!
//! Detects unreachable code by walking SIR function bodies.
//!
//! This detector finds:
//! - Statements after return/revert (unreachable code)

use crate::detector::id::DetectorId;
use crate::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use analysis::context::AnalysisContext;
use analysis::pass::Pass;
use analysis::pass::meta::PassLevel;
use analysis::pass::meta::PassRepresentation;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;
use scirs::sir::{Decl, MemberDecl, Stmt};
use std::any::TypeId;

/// SIR structural detector for dead code (unreachable statements).
#[derive(Debug, Default)]
pub struct DeadCodeSirDetector;

impl DeadCodeSirDetector {
    pub fn new() -> Self {
        Self
    }

    /// Check a list of sequential statements for unreachable code after
    /// a terminator (`return`, `revert`, `break`, `continue`).
    fn check_stmts(
        &self,
        stmts: &[Stmt],
        contract_name: &str,
        func_name: &str,
        bugs: &mut Vec<Bug>,
    ) {
        let mut found_terminator = false;

        for stmt in stmts {
            if found_terminator {
                bugs.push(Bug::new(
                    self.name(),
                    Some(&format!(
                        "Unreachable code in '{}.{}': statement after return/revert.",
                        contract_name, func_name,
                    )),
                    Loc::new(0, 0, 0, 0),
                    self.bug_kind(),
                    self.bug_category(),
                    self.risk_level(),
                    self.cwe_ids(),
                    self.swc_ids(),
                ));
                // Only report the first unreachable statement per block.
                break;
            }

            if self.is_terminator(stmt) {
                found_terminator = true;
            }

            // Recurse into compound statements.
            self.check_stmt_recursively(stmt, contract_name, func_name, bugs);
        }
    }

    fn check_stmt_recursively(
        &self,
        stmt: &Stmt,
        contract_name: &str,
        func_name: &str,
        bugs: &mut Vec<Bug>,
    ) {
        match stmt {
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

    fn is_terminator(&self, stmt: &Stmt) -> bool {
        matches!(stmt, Stmt::Return(_) | Stmt::Revert(_) | Stmt::Break | Stmt::Continue)
    }
}

impl Pass for DeadCodeSirDetector {
    fn name(&self) -> &'static str {
        "Dead Code"
    }

    fn description(&self) -> &'static str {
        "Detects unreachable code after return/revert using SIR tree walking"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ir
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }
}

impl BugDetectionPass for DeadCodeSirDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::DeadCode
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
        vec![561] // CWE-561: Dead Code
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![]
    }

    fn recommendation(&self) -> &'static str {
        "Remove unreachable code and unused functions to improve code clarity \
         and reduce gas costs during deployment."
    }

    fn references(&self) -> Vec<&'static str> {
        vec!["https://cwe.mitre.org/data/definitions/561.html"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dead_code_detector() {
        let detector = DeadCodeSirDetector::new();
        assert_eq!(detector.detector_id(), DetectorId::DeadCode);
        assert_eq!(detector.risk_level(), RiskLevel::Low);
    }
}
