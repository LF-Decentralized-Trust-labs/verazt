//! Constant/Immutable State Variable detector.
//!
//! Detects state variables that could be constant or immutable.

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::detectors::{Detector, ConfidenceLevel, create_bug};
use crate::engine::context::AnalysisContext;
use crate::passes::PassId;
use solidity::ast::{
    ContractDef, ContractElem, Loc, SourceUnit, SourceUnitElem, VarDecl, VarMut,
};
use std::collections::HashSet;

/// Detector for state variables that could be constant or immutable.
pub struct ConstantStateVarDetector;

impl ConstantStateVarDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ConstantStateVarDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for ConstantStateVarDetector {
    fn id(&self) -> &'static str {
        "constant-state-var"
    }

    fn name(&self) -> &'static str {
        "Constant/Immutable State Variable"
    }

    fn description(&self) -> &'static str {
        "State variables that are never modified after initialization should be \
         declared as constant or immutable to save gas."
    }

    fn required_passes(&self) -> Vec<PassId> {
        vec![PassId::SymbolTable, PassId::StateMutation]
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

    fn detect(&self, context: &AnalysisContext) -> Vec<Bug> {
        let mut bugs = Vec::new();

        for source_unit in &context.source_units {
            self.visit_source_unit(source_unit, context, &mut bugs);
        }

        bugs
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

impl ConstantStateVarDetector {
    fn visit_source_unit(&self, source_unit: &SourceUnit, context: &AnalysisContext, bugs: &mut Vec<Bug>) {
        for elem in &source_unit.elems {
            if let SourceUnitElem::Contract(contract) = elem {
                self.visit_contract(contract, context, bugs);
            }
        }
    }

    fn visit_contract(&self, contract: &ContractDef, context: &AnalysisContext, bugs: &mut Vec<Bug>) {
        // Collect all state variables that are not already constant/immutable
        let mut mutable_state_vars: Vec<&VarDecl> = Vec::new();

        for elem in &contract.body {
            if let ContractElem::Var(var) = elem {
                // Skip if already constant or immutable
                if matches!(var.mutability, VarMut::Constant | VarMut::Immutable) {
                    continue;
                }
                mutable_state_vars.push(var);
            }
        }

        if mutable_state_vars.is_empty() {
            return;
        }

        // Get the set of modified state variables from context
        let mut modified_vars: HashSet<String> = HashSet::new();

        if let Some(state_mutations) = &context.state_mutations {
            for (_func_id, vars) in &state_mutations.function_writes {
                for var in vars {
                    modified_vars.insert(var.base.to_string());
                }
            }
        }

        // Check for each state variable
        for var in mutable_state_vars {
            let var_name = var.name.base.as_str();

            // If not modified anywhere, suggest constant/immutable
            if !modified_vars.contains(var_name) {
                // Check if it has an initializer (could be constant)
                let can_be_constant = var.value.is_some() && self.is_constant_expr(var.value.as_ref().unwrap());

                let loc = var.loc.unwrap_or(Loc::new(1, 1, 1, 1));

                if can_be_constant {
                    let bug = create_bug(
                        self,
                        Some(&format!(
                            "State variable '{}' is never modified and could be declared as 'constant'.",
                            var_name
                        )),
                        loc,
                    );
                    bugs.push(bug);
                } else {
                    let bug = create_bug(
                        self,
                        Some(&format!(
                            "State variable '{}' is never modified after initialization and could be declared as 'immutable'.",
                            var_name
                        )),
                        loc,
                    );
                    bugs.push(bug);
                }
            }
        }
    }

    fn is_constant_expr(&self, expr: &solidity::ast::Expr) -> bool {
        match expr {
            // Literals are constant
            solidity::ast::Expr::Lit(_) => true,

            // Binary operations on constants are constant
            solidity::ast::Expr::Binary(bin) => {
                self.is_constant_expr(&bin.left) && self.is_constant_expr(&bin.right)
            }

            // Unary operations on constants are constant
            solidity::ast::Expr::Unary(unary) => {
                self.is_constant_expr(&unary.body)
            }

            // Type conversions of constants
            solidity::ast::Expr::Call(call) => {
                // Check if it's a type conversion (e.g., uint256(0))
                if let solidity::ast::Expr::Ident(_) = call.callee.as_ref() {
                    // Simple heuristic: could be type conversion
                    match &call.args {
                        solidity::ast::CallArgs::Unnamed(args) if args.len() == 1 => {
                            self.is_constant_expr(&args[0])
                        }
                        _ => false,
                    }
                } else {
                    false
                }
            }

            // Everything else is not a compile-time constant
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_state_var_detector_new() {
        let detector = ConstantStateVarDetector::new();
        assert_eq!(detector.id(), "constant-state-var");
    }
}
