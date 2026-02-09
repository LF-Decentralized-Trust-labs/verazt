//! Constant State Variable Detector (GREP-based)
//!
//! Detects state variables that could be declared constant or immutable
//! using pattern matching.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::detection::pass::{BugDetectionPass, ConfidenceLevel, DetectorResult, create_bug};
use bugs::bug::{Bug, BugKind, RiskLevel};
use solidity::ast::{Block, ContractDef, ContractElem, Expr, Loc, SourceUnitElem, Stmt, VarMut};
use std::collections::HashSet;

/// GREP-based detector for state variables that could be constant.
///
/// Finds state variables that are initialized but never modified,
/// suggesting they could be declared as `constant` or `immutable`.
#[derive(Debug, Default)]
pub struct ConstantStateVarGrepDetector;

impl ConstantStateVarGrepDetector {
    pub fn new() -> Self {
        Self
    }

    fn check_contract(&self, contract: &ContractDef, bugs: &mut Vec<Bug>) {
        let contract_name = &contract.name.base;

        // Collect all state variables that are initialized and not already
        // constant/immutable
        let mut candidates: Vec<(&str, Option<Loc>)> = Vec::new();
        let mut candidate_names: HashSet<String> = HashSet::new();

        for elem in &contract.body {
            if let ContractElem::Var(state_var) = elem {
                let is_already_constant =
                    matches!(state_var.mutability, VarMut::Constant | VarMut::Immutable);

                if !is_already_constant && state_var.value.is_some() {
                    let name = state_var.name.base.as_str();
                    candidates.push((name, state_var.loc));
                    candidate_names.insert(state_var.name.base.clone());
                }
            }
        }

        if candidates.is_empty() {
            return;
        }

        // Find all state variable names that are assigned to in functions
        let mut assigned_vars: HashSet<String> = HashSet::new();
        for elem in &contract.body {
            if let ContractElem::Func(func) = elem {
                if let Some(body) = &func.body {
                    self.collect_assigned_vars(body, &candidate_names, &mut assigned_vars);
                }
            }
        }

        // Report candidates that are never assigned
        for (name, loc) in &candidates {
            if !assigned_vars.contains(*name) {
                let loc = loc.unwrap_or(Loc::new(1, 1, 1, 1));
                let bug = create_bug(
                    self,
                    Some(&format!(
                        "State variable '{}' in '{}' is never modified after initialization. \
                         Consider declaring it as 'constant' or 'immutable' to save gas.",
                        name, contract_name,
                    )),
                    loc,
                );
                bugs.push(bug);
            }
        }
    }

    fn collect_assigned_vars(
        &self,
        block: &Block,
        candidates: &HashSet<String>,
        assigned: &mut HashSet<String>,
    ) {
        for stmt in &block.body {
            self.collect_from_stmt(stmt, candidates, assigned);
        }
    }

    fn collect_from_stmt(
        &self,
        stmt: &Stmt,
        candidates: &HashSet<String>,
        assigned: &mut HashSet<String>,
    ) {
        match stmt {
            Stmt::Expr(expr_stmt) => {
                if let Expr::Assign(assign) = &expr_stmt.expr {
                    self.collect_assigned_name(&assign.left, candidates, assigned);
                }
            }
            Stmt::Block(block) => {
                self.collect_assigned_vars(block, candidates, assigned);
            }
            Stmt::If(if_stmt) => {
                self.collect_from_stmt(&if_stmt.true_branch, candidates, assigned);
                if let Some(false_br) = &if_stmt.false_branch {
                    self.collect_from_stmt(false_br, candidates, assigned);
                }
            }
            Stmt::While(w) => {
                self.collect_from_stmt(&w.body, candidates, assigned);
            }
            Stmt::DoWhile(d) => {
                self.collect_from_stmt(&d.body, candidates, assigned);
            }
            Stmt::For(f) => {
                if let Some(pre) = &f.pre_loop {
                    self.collect_from_stmt(pre, candidates, assigned);
                }
                self.collect_from_stmt(&f.body, candidates, assigned);
            }
            Stmt::Try(t) => {
                self.collect_assigned_vars(&t.body, candidates, assigned);
                for catch in &t.catch_clauses {
                    self.collect_assigned_vars(&catch.body, candidates, assigned);
                }
            }
            _ => {}
        }
    }

    fn collect_assigned_name(
        &self,
        expr: &Expr,
        candidates: &HashSet<String>,
        assigned: &mut HashSet<String>,
    ) {
        match expr {
            Expr::Ident(ident) => {
                let name = &ident.name.base;
                if candidates.contains(name.as_str()) {
                    assigned.insert(name.clone());
                }
            }
            Expr::Member(m) => {
                self.collect_assigned_name(&m.base, candidates, assigned);
            }
            Expr::Index(i) => {
                self.collect_assigned_name(&i.base_expr, candidates, assigned);
            }
            _ => {}
        }
    }
}

impl Pass for ConstantStateVarGrepDetector {
    fn id(&self) -> PassId {
        PassId::ConstantStateVar
    }

    fn name(&self) -> &'static str {
        "Constant State Variable"
    }

    fn description(&self) -> &'static str {
        "Detects state variables that could be constant/immutable"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ast
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![]
    }
}

impl BugDetectionPass for ConstantStateVarGrepDetector {
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
        BugKind::Optimization
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
        vec![]
    }

    fn recommendation(&self) -> &'static str {
        "Use 'constant' for compile-time constants and 'immutable' for values \
         set once in the constructor. This saves gas by not using storage."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://docs.soliditylang.org/en/latest/contracts.html#constant-and-immutable-state-variables",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_state_var_grep_detector() {
        let detector = ConstantStateVarGrepDetector::new();
        assert_eq!(detector.id(), PassId::ConstantStateVar);
        assert_eq!(detector.risk_level(), RiskLevel::Low);
    }
}
