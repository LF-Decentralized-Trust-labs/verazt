//! Shadowing detector.
//!
//! Detects variable shadowing (SWC-119).

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::detectors::{Detector, ConfidenceLevel, create_bug};
use crate::engine::context::AnalysisContext;
use crate::passes::PassId;
use solidity::ast::{ContractDef, ContractElem, Loc, Stmt, VarDecl};
use std::collections::HashMap;

/// Detector for variable shadowing.
pub struct ShadowingDetector;

impl ShadowingDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ShadowingDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for ShadowingDetector {
    fn id(&self) -> &'static str {
        "shadowing"
    }

    fn name(&self) -> &'static str {
        "Variable Shadowing"
    }

    fn description(&self) -> &'static str {
        "Local variables or function parameters shadow state variables or \
         inherited variables, which can lead to confusion and bugs."
    }

    fn required_passes(&self) -> Vec<PassId> {
        vec![PassId::SymbolTable]
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Medium
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

    fn detect(&self, context: &AnalysisContext) -> Vec<Bug> {
        let mut bugs = Vec::new();
        
        for source_unit in &context.source_units {
            for elem in &source_unit.elems {
                if let solidity::ast::SourceUnitElem::Contract(contract) = elem {
                    self.check_contract(&contract.name.base, &contract, &mut bugs);
                }
            }
        }
        
        bugs
    }

    fn recommendation(&self) -> &'static str {
        "Rename local variables or parameters to avoid shadowing state variables. \
         Use descriptive names that clearly indicate the variable's scope."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-119",
        ]
    }
}

impl ShadowingDetector {
    fn check_contract(&self, contract_name: &str, contract: &ContractDef, bugs: &mut Vec<Bug>) {
        // Collect state variables
        let mut state_vars: HashMap<String, Loc> = HashMap::new();
        
        for elem in &contract.body {
            if let ContractElem::Var(state_var) = elem {
                state_vars.insert(
                    state_var.name.base.clone(),
                    state_var.loc.unwrap_or(Loc::new(1, 1, 1, 1)),
                );
            }
        }
        
        // Check functions for shadowing
        for elem in &contract.body {
            if let ContractElem::Func(func) = elem {
                // Check function parameters
                for param in &func.params {
                    if let Some(state_loc) = state_vars.get(&param.name.base) {
                        let loc = param.loc.unwrap_or(Loc::new(1, 1, 1, 1));
                        let bug = create_bug(
                            self,
                            Some(&format!(
                                "Parameter '{}' shadows state variable in contract '{}' \
                                 defined at line {}",
                                param.name.base, contract_name, state_loc.start_line
                            )),
                            loc,
                        );
                        bugs.push(bug);
                    }
                }
                
                // Check function body
                if let Some(body) = &func.body {
                    self.check_block(contract_name, body, &state_vars, bugs);
                }
            }
        }
    }

    fn check_block(
        &self,
        contract_name: &str,
        block: &solidity::ast::Block,
        state_vars: &HashMap<String, Loc>,
        bugs: &mut Vec<Bug>,
    ) {
        for s in &block.body {
            self.check_statement(contract_name, s, state_vars, bugs);
        }
    }

    fn check_statement(
        &self,
        contract_name: &str,
        stmt: &Stmt,
        state_vars: &HashMap<String, Loc>,
        bugs: &mut Vec<Bug>,
    ) {
        match stmt {
            Stmt::Block(block) => {
                self.check_block(contract_name, block, state_vars, bugs);
            }
            Stmt::VarDecl(var_decl) => {
                for var in var_decl.var_decls.iter().flatten() {
                    if let Some(state_loc) = state_vars.get(&var.name.base) {
                        let loc = var.loc.unwrap_or(Loc::new(1, 1, 1, 1));
                        let bug = create_bug(
                            self,
                            Some(&format!(
                                "Local variable '{}' shadows state variable in contract '{}' \
                                 defined at line {}",
                                var.name.base, contract_name, state_loc.start_line
                            )),
                            loc,
                        );
                        bugs.push(bug);
                    }
                }
            }
            Stmt::If(if_stmt) => {
                self.check_statement(contract_name, &if_stmt.true_branch, state_vars, bugs);
                if let Some(false_br) = &if_stmt.false_branch {
                    self.check_statement(contract_name, false_br, state_vars, bugs);
                }
            }
            Stmt::While(while_stmt) => {
                self.check_statement(contract_name, &while_stmt.body, state_vars, bugs);
            }
            Stmt::For(for_stmt) => {
                if let Some(pre) = &for_stmt.pre_loop {
                    self.check_statement(contract_name, pre, state_vars, bugs);
                }
                self.check_statement(contract_name, &for_stmt.body, state_vars, bugs);
            }
            Stmt::DoWhile(do_while) => {
                self.check_statement(contract_name, &do_while.body, state_vars, bugs);
            }
            Stmt::Try(try_stmt) => {
                self.check_block(contract_name, &try_stmt.body, state_vars, bugs);
            }
            _ => {}
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shadowing_detector() {
        let detector = ShadowingDetector::new();
        assert_eq!(detector.id(), "shadowing");
        assert_eq!(detector.swc_ids(), vec![119]);
    }
}
