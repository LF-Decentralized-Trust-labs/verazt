//! Dead Code Detector
//!
//! Detects unreachable code by walking SIR function bodies.

use crate::detectors::sir::detector::{DetectionLevel, ScanDetector};
use crate::detectors::{ConfidenceLevel, DetectorId, DetectorMeta, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::{ContractDecl, MemberDecl, Module, Stmt};

const META: DetectorMeta = DetectorMeta {
    bug_category: BugCategory::CodeQuality,
    bug_kind: BugKind::Refactoring,
    confidence: ConfidenceLevel::High,
    cwe_ids: &[561],
    description: "Detects unreachable code after return/revert using SIR tree walking",
    id: DetectorId::DeadCode,
    name: "Dead Code",
    recommendation: "Remove unreachable code and unused functions to improve code clarity \
         and reduce gas costs during deployment.",
    references: &["https://cwe.mitre.org/data/definitions/561.html"],
    risk_level: RiskLevel::Low,
    swc_ids: &[],
    target: Target::Evm,
};

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
                bugs.push(META.bug(
                    Some(&format!(
                        "Unreachable code in '{}.{}': statement after return/revert.",
                        contract_name, func_name,
                    )),
                    stmt.span().cloned().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
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
    fn meta(&self) -> &'static DetectorMeta {
        &META
    }

    fn level(&self) -> DetectionLevel {
        DetectionLevel::Contract
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
        assert_eq!(detector.meta().id, DetectorId::DeadCode);
        assert_eq!(detector.meta().risk_level, RiskLevel::Low);
    }
}
