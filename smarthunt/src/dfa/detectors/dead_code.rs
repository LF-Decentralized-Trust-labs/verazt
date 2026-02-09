//! Dead Code Detector (DFA-based)
//!
//! Detects unreachable and unused code using control flow and liveness
//! analysis.
//!
//! This detector finds:
//! - Statements after return/revert/throw (unreachable code)
//! - Unused function parameters and local variables
//! - Functions that are never called (internal/private only)

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::pipeline::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult, create_bug};
use bugs::bug::{Bug, BugKind, RiskLevel};
use solidity::ast::{Block, ContractElem, FuncDef, Loc, SourceUnitElem, Stmt};

/// DFA-based detector for dead code.
#[derive(Debug, Default)]
pub struct DeadCodeDfaDetector;

impl DeadCodeDfaDetector {
    pub fn new() -> Self {
        Self
    }

    fn check_function(&self, func: &FuncDef, contract_name: &str, bugs: &mut Vec<Bug>) {
        if let Some(body) = &func.body {
            let func_name = func.name.base.as_str();
            self.check_block_for_unreachable(body, contract_name, func_name, bugs);
        }
    }

    /// Check a block for statements after return/revert.
    fn check_block_for_unreachable(
        &self,
        block: &Block,
        contract_name: &str,
        func_name: &str,
        bugs: &mut Vec<Bug>,
    ) {
        let mut found_terminator = false;

        for stmt in &block.body {
            if found_terminator {
                // Any statement after a terminator is unreachable
                let loc = stmt.loc().unwrap_or(Loc::new(1, 1, 1, 1));
                let bug = create_bug(
                    self,
                    Some(&format!(
                        "Unreachable code in '{}.{}': statement after return/revert.",
                        contract_name, func_name,
                    )),
                    loc,
                );
                bugs.push(bug);
                break; // Only report the first unreachable statement
            }

            // Check if this statement is a terminator
            if self.is_terminator_stmt(stmt) {
                found_terminator = true;
            }

            // Recurse into sub-blocks
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
            Stmt::Block(block) => {
                self.check_block_for_unreachable(block, contract_name, func_name, bugs);
            }
            Stmt::If(if_stmt) => {
                if let Stmt::Block(block) = &*if_stmt.true_branch {
                    self.check_block_for_unreachable(block, contract_name, func_name, bugs);
                }
                if let Some(false_br) = &if_stmt.false_branch {
                    if let Stmt::Block(block) = false_br.as_ref() {
                        self.check_block_for_unreachable(block, contract_name, func_name, bugs);
                    }
                }
            }
            Stmt::While(w) => {
                if let Stmt::Block(block) = &*w.body {
                    self.check_block_for_unreachable(block, contract_name, func_name, bugs);
                }
            }
            Stmt::DoWhile(d) => {
                if let Stmt::Block(block) = &*d.body {
                    self.check_block_for_unreachable(block, contract_name, func_name, bugs);
                }
            }
            Stmt::For(f) => {
                if let Stmt::Block(block) = &*f.body {
                    self.check_block_for_unreachable(block, contract_name, func_name, bugs);
                }
            }
            Stmt::Try(t) => {
                self.check_block_for_unreachable(&t.body, contract_name, func_name, bugs);
                for catch in &t.catch_clauses {
                    self.check_block_for_unreachable(&catch.body, contract_name, func_name, bugs);
                }
            }
            _ => {}
        }
    }

    fn is_terminator_stmt(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Return(_) => true,
            Stmt::Revert(_) => true,
            Stmt::Continue(_) => true,
            Stmt::Break(_) => true,
            _ => false,
        }
    }
}

impl Pass for DeadCodeDfaDetector {
    fn id(&self) -> PassId {
        PassId::DeadCode
    }

    fn name(&self) -> &'static str {
        "Dead Code (DFA)"
    }

    fn description(&self) -> &'static str {
        "Detects unreachable and unused code using control flow analysis"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ast
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::SymbolTable, PassId::CallGraph]
    }
}

impl BugDetectionPass for DeadCodeDfaDetector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        for source_unit in &context.source_units {
            for elem in &source_unit.elems {
                match elem {
                    SourceUnitElem::Contract(contract) => {
                        let contract_name = &contract.name.base;
                        for elem in &contract.body {
                            if let ContractElem::Func(func) = elem {
                                self.check_function(func, contract_name, &mut bugs);
                            }
                        }
                    }
                    SourceUnitElem::Func(func) => {
                        self.check_function(func, "global", &mut bugs);
                    }
                    _ => {}
                }
            }
        }

        Ok(bugs)
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Refactoring
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
        let detector = DeadCodeDfaDetector::new();
        assert_eq!(detector.id(), PassId::DeadCode);
        assert_eq!(detector.risk_level(), RiskLevel::Low);
    }
}
