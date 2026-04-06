//! Dead Code Detector
//!
//! Detects unreachable code by walking SIR function bodies.

use crate::detector::{Confidence, DetectionLevel, ScanDetector, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::{ContractDecl, MemberDecl, Module, Stmt};

/// Scan detector for dead code (unreachable statements).
#[derive(Debug, Default)]
pub struct DeadCodeDetector;

impl DeadCodeDetector {
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
                    stmt.span().cloned().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                    self.bug_kind(),
                    self.bug_category(),
                    self.risk_level(),
                    self.cwe_ids(),
                    self.swc_ids(),
                    Some(self.recommendation()),
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

impl ScanDetector for DeadCodeDetector {
    fn id(&self) -> &'static str {
        "dead-code"
    }

    fn name(&self) -> &'static str {
        "Dead Code"
    }

    fn description(&self) -> &'static str {
        "Detects unreachable code after return/revert using SIR tree walking"
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
        DetectionLevel::Contract
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![561]
    }

    fn recommendation(&self) -> &'static str {
        "Remove unreachable code and unused functions to improve code clarity \
         and reduce gas costs during deployment."
    }

    fn references(&self) -> Vec<&'static str> {
        vec!["https://cwe.mitre.org/data/definitions/561.html"]
    }

    fn check_contract(&self, contract: &ContractDecl, _module: &Module) -> Vec<Bug> {
        let mut bugs = Vec::new();

        for member in &contract.members {
            if let MemberDecl::Function(func) = member {
                if let Some(body) = &func.body {
                    self.check_stmts(body, &contract.name, &func.name, &mut bugs);
                }
            }
        }

        bugs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dead_code_detector() {
        let detector = DeadCodeDetector::new();
        assert_eq!(detector.id(), "dead-code");
        assert_eq!(detector.risk_level(), RiskLevel::Low);
    }
}
