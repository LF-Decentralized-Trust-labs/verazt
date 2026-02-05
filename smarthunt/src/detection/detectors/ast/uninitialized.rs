//! Uninitialized Variable Detector
//!
//! Detects uninitialized storage variables and storage pointers (SWC-109).
//!
//! This detector finds:
//! - State variables that are not explicitly initialized
//! - Local storage pointers that could point to arbitrary storage locations

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use solidity::ast::{ContractDef, ContractElem, DataLoc, Expr, Loc, SourceUnitElem, Stmt, Type, VarMut};
use crate::detection::pass::{BugDetectionPass, ConfidenceLevel, DetectorResult, create_bug};
use std::collections::HashSet;

/// Detector for uninitialized storage variables and pointers.
#[derive(Debug, Default)]
pub struct UninitializedStorageDetector;

impl UninitializedStorageDetector {
    pub fn new() -> Self {
        Self
    }

    fn should_warn_uninitialized(&self, typ: &Type) -> bool {
        // Warn for mapping and array types that might need initialization
        matches!(typ, Type::Mapping(_) | Type::Array(_))
    }

    fn check_contract(&self, contract_name: &str, contract: &ContractDef, bugs: &mut Vec<Bug>) {
        // Check state variables without initialization
        for elem in &contract.body {
            if let ContractElem::Var(state_var) = elem {
                // Skip constants and immutables (they must be initialized)
                let is_constant = matches!(state_var.mutability, VarMut::Constant | VarMut::Immutable);

                if !is_constant && state_var.value.is_none() {
                    // Check if it's a complex type that should be initialized
                    if self.should_warn_uninitialized(&state_var.typ) {
                        let loc = state_var.loc.unwrap_or(Loc::new(1, 1, 1, 1));
                        let var_name = state_var.name.base.as_str();
                        let bug = create_bug(
                            self,
                            Some(&format!(
                                "State variable '{}' in contract '{}' is not initialized. \
                                 Consider initializing it explicitly.",
                                var_name, contract_name
                            )),
                            loc,
                        );
                        bugs.push(bug);
                    }
                }
            }
        }

        // Check functions for uninitialized local storage pointers
        for elem in &contract.body {
            if let ContractElem::Func(func) = elem {
                if let Some(body) = &func.body {
                    let mut initialized: HashSet<String> = HashSet::new();
                    self.check_block(contract_name, body, &mut initialized, bugs);
                }
            }
        }
    }

    fn check_block(
        &self,
        contract_name: &str,
        block: &solidity::ast::Block,
        initialized: &mut HashSet<String>,
        bugs: &mut Vec<Bug>,
    ) {
        for s in &block.body {
            self.check_statement(contract_name, s, initialized, bugs);
        }
    }

    fn check_statement(
        &self,
        contract_name: &str,
        stmt: &Stmt,
        initialized: &mut HashSet<String>,
        bugs: &mut Vec<Bug>,
    ) {
        match stmt {
            Stmt::Block(block) => {
                self.check_block(contract_name, block, initialized, bugs);
            }
            Stmt::VarDecl(var_decl) => {
                for var in var_decl.var_decls.iter().flatten() {
                    // Check for storage location in local variables
                    if let Some(DataLoc::Storage) = &var.data_loc {
                        if var_decl.value.is_none() {
                            let loc = var.loc.unwrap_or(Loc::new(1, 1, 1, 1));
                            let var_name = var.name.base.as_str();
                            let bug = create_bug(
                                self,
                                Some(&format!(
                                    "Local storage pointer '{}' in contract '{}' is not initialized. \
                                     This can lead to unexpected storage access.",
                                    var_name, contract_name
                                )),
                                loc,
                            );
                            bugs.push(bug);
                        }
                    }

                    // Track initialized variables
                    if var_decl.value.is_some() {
                        initialized.insert(var.name.base.clone());
                    }
                }
            }
            Stmt::If(if_stmt) => {
                self.check_statement(contract_name, &if_stmt.true_branch, initialized, bugs);
                if let Some(false_br) = &if_stmt.false_branch {
                    self.check_statement(contract_name, false_br, initialized, bugs);
                }
            }
            Stmt::While(while_stmt) => {
                self.check_statement(contract_name, &while_stmt.body, initialized, bugs);
            }
            Stmt::For(for_stmt) => {
                if let Some(pre) = &for_stmt.pre_loop {
                    self.check_statement(contract_name, pre, initialized, bugs);
                }
                self.check_statement(contract_name, &for_stmt.body, initialized, bugs);
            }
            Stmt::DoWhile(do_while) => {
                self.check_statement(contract_name, &do_while.body, initialized, bugs);
            }
            Stmt::Try(try_stmt) => {
                self.check_block(contract_name, &try_stmt.body, initialized, bugs);
            }
            Stmt::Expr(expr_stmt) => {
                // Check for assignments
                if let Expr::Assign(assign) = &expr_stmt.expr {
                    if let Expr::Ident(ident) = &*assign.left {
                        initialized.insert(ident.name.base.clone());
                    }
                }
            }
            _ => {}
        }
    }
}

impl Pass for UninitializedStorageDetector {
    fn id(&self) -> PassId {
        PassId::UninitializedStorage
    }

    fn name(&self) -> &'static str {
        "Uninitialized Storage"
    }

    fn description(&self) -> &'static str {
        "Detects uninitialized storage variables and storage pointers"
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

impl BugDetectionPass for UninitializedStorageDetector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        for source_unit in &context.source_units {
            for elem in &source_unit.elems {
                if let SourceUnitElem::Contract(contract) = elem {
                    self.check_contract(&contract.name.base, contract, &mut bugs);
                }
            }
        }

        Ok(bugs)
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::High
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::Medium
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![824] // CWE-824: Access of Uninitialized Pointer
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![109] // SWC-109: Uninitialized Storage Pointer
    }

    fn recommendation(&self) -> &'static str {
        "Initialize all storage variables explicitly. For local variables with storage \
         location, assign a reference to a state variable before use."
    }

    fn references(&self) -> Vec<&'static str> {
        vec!["https://swcregistry.io/docs/SWC-109"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uninitialized_storage_detector() {
        let detector = UninitializedStorageDetector::new();
        assert_eq!(detector.id(), PassId::UninitializedStorage);
        assert_eq!(detector.swc_ids(), vec![109]);
        assert_eq!(detector.risk_level(), RiskLevel::High);
    }
}
