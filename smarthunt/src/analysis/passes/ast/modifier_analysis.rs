//! Modifier Analysis Pass
//!
//! This pass analyzes function modifiers and their usage.

use crate::analysis::pass::{Pass, AnalysisPass, PassResult};
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::analysis::context::AnalysisContext;
use crate::analysis::passes::ast::symbol_table::FunctionId;
use solidity::ast::{
    ContractDef, ContractElem, FuncDef, FuncKind, Name, SourceUnit, SourceUnitElem,
};
use std::collections::{HashMap, HashSet};

/// Information about a modifier.
#[derive(Debug, Clone)]
pub struct ModifierInfo {
    /// Modifier name.
    pub name: Name,
    /// Contract where defined.
    pub contract: Name,
    /// Whether this is an access control modifier.
    pub is_access_control: bool,
    /// Whether this is a reentrancy guard.
    pub is_reentrancy_guard: bool,
    /// Location in source.
    pub loc: Option<solidity::ast::Loc>,
}

/// Modifier usage on a function.
#[derive(Debug, Clone)]
pub struct ModifierUsage {
    /// Function using the modifier.
    pub function: FunctionId,
    /// Modifier name.
    pub modifier_name: Name,
    /// Arguments passed to modifier.
    pub has_args: bool,
}

/// Modifier analysis results.
#[derive(Debug, Clone, Default)]
pub struct ModifierAnalysis {
    /// All modifier definitions indexed by (contract, name).
    pub definitions: HashMap<(Name, Name), ModifierInfo>,

    /// Modifier usages by function.
    pub function_modifiers: HashMap<FunctionId, Vec<Name>>,

    /// Functions using each modifier.
    pub modifier_functions: HashMap<(Name, Name), Vec<FunctionId>>,

    /// Access control modifier names.
    pub access_control_modifiers: HashSet<String>,

    /// Reentrancy guard modifier names.
    pub reentrancy_guard_modifiers: HashSet<String>,
}

impl ModifierAnalysis {
    /// Create a new empty modifier analysis.
    pub fn new() -> Self {
        let mut analysis = Self::default();
        
        // Common access control modifier patterns
        analysis.access_control_modifiers.insert("onlyOwner".to_string());
        analysis.access_control_modifiers.insert("onlyAdmin".to_string());
        analysis.access_control_modifiers.insert("onlyRole".to_string());
        analysis.access_control_modifiers.insert("onlyMinter".to_string());
        analysis.access_control_modifiers.insert("onlyPauser".to_string());
        analysis.access_control_modifiers.insert("onlyGovernance".to_string());
        analysis.access_control_modifiers.insert("onlyController".to_string());
        analysis.access_control_modifiers.insert("onlyAuthorized".to_string());
        analysis.access_control_modifiers.insert("whenNotPaused".to_string());
        analysis.access_control_modifiers.insert("whenPaused".to_string());
        
        // Common reentrancy guard patterns
        analysis.reentrancy_guard_modifiers.insert("nonReentrant".to_string());
        analysis.reentrancy_guard_modifiers.insert("noReentrant".to_string());
        analysis.reentrancy_guard_modifiers.insert("reentrancyGuard".to_string());
        
        analysis
    }

    /// Build modifier analysis from source units.
    pub fn from_source_units(source_units: &[SourceUnit]) -> Self {
        let mut analysis = Self::new();

        for source_unit in source_units {
            for elem in &source_unit.elems {
                if let SourceUnitElem::Contract(contract) = elem {
                    analysis.process_contract(contract);
                }
            }
        }

        analysis
    }

    fn process_contract(&mut self, contract: &ContractDef) {
        for elem in &contract.body {
            if let ContractElem::Func(func) = elem {
                // Check if this is a modifier definition
                if func.kind == FuncKind::Modifier {
                    self.add_modifier(contract, func);
                } else {
                    self.process_function(contract, func);
                }
            }
        }
    }

    fn add_modifier(&mut self, contract: &ContractDef, modifier: &FuncDef) {
        let name_str = modifier.name.base.as_str();
        
        let is_access_control = self.access_control_modifiers.contains(name_str)
            || name_str.starts_with("only")
            || name_str.starts_with("when");
            
        let is_reentrancy_guard = self.reentrancy_guard_modifiers.contains(name_str)
            || name_str.contains("reentran");

        let info = ModifierInfo {
            name: modifier.name.clone(),
            contract: contract.name.clone(),
            is_access_control,
            is_reentrancy_guard,
            loc: modifier.loc,
        };

        self.definitions.insert(
            (contract.name.clone(), modifier.name.clone()),
            info,
        );
    }

    fn process_function(&mut self, contract: &ContractDef, func: &FuncDef) {
        let func_id = FunctionId {
            contract: Some(contract.name.clone()),
            name: func.name.clone(),
            kind: func.kind.clone(),
        };

        // Extract modifier names from modifier invocations (CallExpr)
        let modifiers: Vec<Name> = func
            .modifier_invocs
            .iter()
            .filter_map(|m| {
                // Extract name from callee expression
                match m.callee.as_ref() {
                    solidity::ast::Expr::Ident(ident) => Some(ident.name.clone()),
                    _ => None,
                }
            })
            .collect();

        if !modifiers.is_empty() {
            self.function_modifiers.insert(func_id.clone(), modifiers.clone());

            for modifier_name in &modifiers {
                self.modifier_functions
                    .entry((contract.name.clone(), modifier_name.clone()))
                    .or_default()
                    .push(func_id.clone());
            }
        }
    }

    /// Get modifiers used by a function.
    pub fn get_function_modifiers(&self, func: &FunctionId) -> Vec<Name> {
        self.function_modifiers.get(func).cloned().unwrap_or_default()
    }

    /// Check if a function has a specific modifier.
    pub fn has_modifier(&self, func: &FunctionId, modifier: &Name) -> bool {
        self.function_modifiers
            .get(func)
            .map(|mods| mods.contains(modifier))
            .unwrap_or(false)
    }

    /// Check if a function has any access control modifier.
    pub fn has_access_control(&self, func: &FunctionId) -> bool {
        if let Some(modifiers) = self.function_modifiers.get(func) {
            for modifier in modifiers {
                if self.access_control_modifiers.contains(&modifier.base) {
                    return true;
                }
                // Check if the modifier is defined as access control
                if let Some(contract) = &func.contract {
                    if let Some(info) = self.definitions.get(&(contract.clone(), modifier.clone())) {
                        if info.is_access_control {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Check if a function has a reentrancy guard.
    pub fn has_reentrancy_guard(&self, func: &FunctionId) -> bool {
        if let Some(modifiers) = self.function_modifiers.get(func) {
            for modifier in modifiers {
                if self.reentrancy_guard_modifiers.contains(&modifier.base) {
                    return true;
                }
                // Check if the modifier is defined as reentrancy guard
                if let Some(contract) = &func.contract {
                    if let Some(info) = self.definitions.get(&(contract.clone(), modifier.clone())) {
                        if info.is_reentrancy_guard {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Get all access control modifiers.
    pub fn get_access_control_modifiers(&self) -> Vec<&ModifierInfo> {
        self.definitions
            .values()
            .filter(|m| m.is_access_control)
            .collect()
    }

    /// Get all reentrancy guard modifiers.
    pub fn get_reentrancy_guards(&self) -> Vec<&ModifierInfo> {
        self.definitions
            .values()
            .filter(|m| m.is_reentrancy_guard)
            .collect()
    }
}

/// Pass for analyzing modifiers.
pub struct ModifierAnalysisPass;

impl ModifierAnalysisPass {
    /// Create a new modifier analysis pass.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ModifierAnalysisPass {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for ModifierAnalysisPass {
    fn id(&self) -> PassId {
        PassId::ModifierAnalysis
    }

    fn name(&self) -> &'static str {
        "Modifier Analysis"
    }

    fn description(&self) -> &'static str {
        "Analyzes function modifiers including access control and reentrancy guards"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Function
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ast
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::SymbolTable]
    }
}

impl AnalysisPass for ModifierAnalysisPass {
    fn run(&self, context: &mut AnalysisContext) -> PassResult<()> {
        let analysis = ModifierAnalysis::from_source_units(&context.source_units);
        context.store_artifact("modifier_analysis", analysis);
        context.record_ast_traversal();
        Ok(())
    }

    fn is_completed(&self, context: &AnalysisContext) -> bool {
        context.has_artifact("modifier_analysis")
    }
}

/// Convenience trait to get modifier analysis from context.
pub trait ModifierAnalysisExt {
    fn modifier_analysis(&self) -> Option<&ModifierAnalysis>;
}

impl ModifierAnalysisExt for AnalysisContext {
    fn modifier_analysis(&self) -> Option<&ModifierAnalysis> {
        self.get_artifact::<ModifierAnalysis>("modifier_analysis")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modifier_analysis_pass() {
        let pass = ModifierAnalysisPass::new();
        assert_eq!(pass.id(), PassId::ModifierAnalysis);
        assert_eq!(pass.dependencies(), vec![PassId::SymbolTable]);
    }
}
