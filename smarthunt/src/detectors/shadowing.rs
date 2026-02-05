//! Shadowing detector.
//!
//! Detects variable shadowing (SWC-119).

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::detectors::{Detector, ConfidenceLevel, create_bug};
use crate::engine::context::AnalysisContext;
use crate::passes::PassId;
use solidity::ast::{ContractDef, ContractElem, Loc};
use solidity::ast::utils::Visit;
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
        let mut visitor = ShadowingVisitor::new(self);
        for source_unit in &context.source_units {
            visitor.visit_source_unit(source_unit);
        }
        visitor.bugs
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

impl ShadowingDetector {}

/// Visitor to collect variable shadowing bugs.
struct ShadowingVisitor<'a, 'b> {
    detector: &'a ShadowingDetector,
    bugs: Vec<Bug>,
    contract_name: Option<String>,
    state_vars: HashMap<String, Loc>,
    _marker: std::marker::PhantomData<&'b ()>,
}

impl<'a, 'b> ShadowingVisitor<'a, 'b> {
    fn new(detector: &'a ShadowingDetector) -> Self {
        Self {
            detector,
            bugs: Vec::new(),
            contract_name: None,
            state_vars: HashMap::new(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, 'b> Visit<'b> for ShadowingVisitor<'a, 'b> {
    fn visit_contract_def(&mut self, contract: &'b ContractDef) {
        self.contract_name = Some(contract.name.base.clone());
        self.state_vars.clear();

        // Collect state variables first
        for elem in &contract.body {
            if let ContractElem::Var(state_var) = elem {
                self.state_vars.insert(
                    state_var.name.base.clone(),
                    state_var.loc.unwrap_or(Loc::new(1, 1, 1, 1)),
                );
            }
        }

        // Now visit the contract
        solidity::ast::utils::visit::default::visit_contract_def(self, contract);
        self.contract_name = None;
        self.state_vars.clear();
    }

    fn visit_func_def(&mut self, func: &'b solidity::ast::FuncDef) {
        let contract_name = self.contract_name.as_deref().unwrap_or("<unknown>");

        // Check function parameters
        for param in &func.params {
            if let Some(state_loc) = self.state_vars.get(&param.name.base) {
                let loc = param.loc.unwrap_or(Loc::new(1, 1, 1, 1));
                let bug = create_bug(
                    self.detector,
                    Some(&format!(
                        "Parameter '{}' shadows state variable in contract '{}' \
                         defined at line {}",
                        param.name.base, contract_name, state_loc.start_line
                    )),
                    loc,
                );
                self.bugs.push(bug);
            }
        }

        solidity::ast::utils::visit::default::visit_func_def(self, func);
    }

    fn visit_var_decl_stmt(&mut self, stmt: &'b solidity::ast::VarDeclStmt) {
        let contract_name = self.contract_name.as_deref().unwrap_or("<unknown>");

        for var in stmt.var_decls.iter().flatten() {
            if let Some(state_loc) = self.state_vars.get(&var.name.base) {
                let loc = var.loc.unwrap_or(Loc::new(1, 1, 1, 1));
                let bug = create_bug(
                    self.detector,
                    Some(&format!(
                        "Local variable '{}' shadows state variable in contract '{}' \
                         defined at line {}",
                        var.name.base, contract_name, state_loc.start_line
                    )),
                    loc,
                );
                self.bugs.push(bug);
            }
        }

        solidity::ast::utils::visit::default::visit_var_decl_stmt(self, stmt);
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
