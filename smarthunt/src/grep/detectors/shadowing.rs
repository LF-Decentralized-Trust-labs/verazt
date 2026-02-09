//! Variable Shadowing Detector (GREP-based)
//!
//! Detects variable shadowing across scopes and inheritance
//! using pattern matching on the AST.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::detection::pass::{BugDetectionPass, ConfidenceLevel, DetectorResult, create_bug};
use bugs::bug::{Bug, BugKind, RiskLevel};
use solidity::ast::{Block, ContractDef, ContractElem, FuncDef, Loc, SourceUnitElem, Stmt};
use std::collections::HashSet;

/// GREP-based detector for variable shadowing.
///
/// Detects local variables that shadow state variables in the same contract.
#[derive(Debug, Default)]
pub struct ShadowingGrepDetector;

impl ShadowingGrepDetector {
    pub fn new() -> Self {
        Self
    }

    fn check_contract(&self, contract: &ContractDef, bugs: &mut Vec<Bug>) {
        // Collect state variable names
        let mut state_vars = HashSet::new();
        for elem in &contract.body {
            if let ContractElem::Var(var) = elem {
                state_vars.insert(var.name.base.clone());
            }
        }

        if state_vars.is_empty() {
            return;
        }

        // Check functions for local variable declarations that shadow state vars
        for elem in &contract.body {
            if let ContractElem::Func(func) = elem {
                self.check_function(func, &contract.name.base, &state_vars, bugs);
            }
        }
    }

    fn check_function(
        &self,
        func: &FuncDef,
        contract_name: &str,
        state_vars: &HashSet<String>,
        bugs: &mut Vec<Bug>,
    ) {
        // Check function parameters
        for param in &func.params {
            if state_vars.contains(&param.name.base as &str) {
                let loc = param.loc.unwrap_or(Loc::new(1, 1, 1, 1));
                let bug = create_bug(
                    self,
                    Some(&format!(
                        "Parameter '{}' in '{}.{}' shadows a state variable.",
                        param.name.base, contract_name, func.name.base,
                    )),
                    loc,
                );
                bugs.push(bug);
            }
        }

        // Check function body for local variable declarations
        if let Some(body) = &func.body {
            self.check_block(body, contract_name, &func.name.base, state_vars, bugs);
        }
    }

    fn check_block(
        &self,
        block: &Block,
        contract_name: &str,
        func_name: &str,
        state_vars: &HashSet<String>,
        bugs: &mut Vec<Bug>,
    ) {
        for stmt in &block.body {
            self.check_stmt(stmt, contract_name, func_name, state_vars, bugs);
        }
    }

    fn check_stmt(
        &self,
        stmt: &Stmt,
        contract_name: &str,
        func_name: &str,
        state_vars: &HashSet<String>,
        bugs: &mut Vec<Bug>,
    ) {
        match stmt {
            Stmt::VarDecl(var_decl) => {
                for var in var_decl.var_decls.iter().flatten() {
                    if state_vars.contains(&var.name.base as &str) {
                        let loc = var.loc.unwrap_or(Loc::new(1, 1, 1, 1));
                        let bug = create_bug(
                            self,
                            Some(&format!(
                                "Local variable '{}' in '{}.{}' shadows a state variable.",
                                var.name.base, contract_name, func_name,
                            )),
                            loc,
                        );
                        bugs.push(bug);
                    }
                }
            }
            Stmt::Block(block) => {
                self.check_block(block, contract_name, func_name, state_vars, bugs);
            }
            Stmt::If(if_stmt) => {
                self.check_stmt(&if_stmt.true_branch, contract_name, func_name, state_vars, bugs);
                if let Some(false_br) = &if_stmt.false_branch {
                    self.check_stmt(false_br, contract_name, func_name, state_vars, bugs);
                }
            }
            Stmt::While(w) => {
                self.check_stmt(&w.body, contract_name, func_name, state_vars, bugs);
            }
            Stmt::DoWhile(d) => {
                self.check_stmt(&d.body, contract_name, func_name, state_vars, bugs);
            }
            Stmt::For(f) => {
                if let Some(pre) = &f.pre_loop {
                    self.check_stmt(pre, contract_name, func_name, state_vars, bugs);
                }
                self.check_stmt(&f.body, contract_name, func_name, state_vars, bugs);
            }
            Stmt::Try(t) => {
                self.check_block(&t.body, contract_name, func_name, state_vars, bugs);
                for catch in &t.catch_clauses {
                    self.check_block(&catch.body, contract_name, func_name, state_vars, bugs);
                }
            }
            _ => {}
        }
    }
}

impl Pass for ShadowingGrepDetector {
    fn id(&self) -> PassId {
        PassId::Shadowing
    }

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
        PassRepresentation::Ast
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![]
    }
}

impl BugDetectionPass for ShadowingGrepDetector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        for source_unit in &context.source_units {
            for elem in &source_unit.elems {
                if let SourceUnitElem::Contract(contract) = elem {
                    self.check_contract(contract, &mut bugs);
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
    fn test_shadowing_grep_detector() {
        let detector = ShadowingGrepDetector::new();
        assert_eq!(detector.id(), PassId::Shadowing);
        assert_eq!(detector.swc_ids(), vec![119]);
    }
}
