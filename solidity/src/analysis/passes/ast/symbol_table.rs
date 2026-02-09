//! Symbol Table Pass
//!
//! This pass builds a symbol table for fast lookup of program entities.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::{AnalysisPass, Pass, PassResult};
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::ast::{
    ContractDef, ContractElem, EnumDef, ErrorDef, EventDef, FuncDef, FuncKind, Name, SourceUnit,
    SourceUnitElem, StructDef, TypeDef, VarDecl,
};
use std::collections::HashMap;

/// Unique identifier for a function.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionId {
    /// Contract name (None for free functions).
    pub contract: Option<Name>,
    /// Function name.
    pub name: Name,
    /// Function kind.
    pub kind: FuncKind,
}

impl FunctionId {
    /// Create a new function ID.
    pub fn new(contract: Option<Name>, name: Name, kind: FuncKind) -> Self {
        Self { contract, name, kind }
    }

    /// Create a function ID from a FuncDef.
    pub fn from_func(func: &FuncDef, contract: Option<&ContractDef>) -> Self {
        Self {
            contract: contract.map(|c| c.name.clone()),
            name: func.name.clone(),
            kind: func.kind.clone(),
        }
    }
}

/// Symbol table for fast lookup of program entities.
#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    /// All contracts indexed by name.
    pub contracts: HashMap<Name, ContractDef>,

    /// All functions indexed by FunctionId.
    pub functions: HashMap<FunctionId, FuncDef>,

    /// All state variables indexed by (contract name, var name).
    pub state_variables: HashMap<(Name, Name), VarDecl>,

    /// All structs indexed by name.
    pub structs: HashMap<Name, StructDef>,

    /// All enums indexed by name.
    pub enums: HashMap<Name, EnumDef>,

    /// All events indexed by name.
    pub events: HashMap<Name, EventDef>,

    /// All errors indexed by name.
    pub errors: HashMap<Name, ErrorDef>,

    /// All user-defined types indexed by name.
    pub user_types: HashMap<Name, TypeDef>,

    /// Free functions (not in a contract).
    pub free_functions: HashMap<Name, FuncDef>,
}

impl SymbolTable {
    /// Create a new empty symbol table.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a symbol table from source units.
    pub fn from_source_units(source_units: &[SourceUnit]) -> Self {
        let mut table = Self::new();

        for source_unit in source_units {
            table.process_source_unit(source_unit);
        }

        table
    }

    fn process_source_unit(&mut self, source_unit: &SourceUnit) {
        for elem in &source_unit.elems {
            match elem {
                SourceUnitElem::Contract(contract) => {
                    self.contracts
                        .insert(contract.name.clone(), contract.clone());
                    self.process_contract(contract);
                }
                SourceUnitElem::Func(func) => {
                    self.free_functions.insert(func.name.clone(), func.clone());
                    let id = FunctionId::from_func(func, None);
                    self.functions.insert(id, func.clone());
                }
                SourceUnitElem::Struct(s) => {
                    self.structs.insert(s.name.clone(), s.clone());
                }
                SourceUnitElem::Enum(e) => {
                    self.enums.insert(e.name.clone(), e.clone());
                }
                SourceUnitElem::Error(e) => {
                    self.errors.insert(e.name.clone(), e.clone());
                }
                SourceUnitElem::UserType(t) => {
                    self.user_types.insert(t.name.clone(), t.clone());
                }
                _ => {}
            }
        }
    }

    fn process_contract(&mut self, contract: &ContractDef) {
        for elem in &contract.body {
            match elem {
                ContractElem::Func(func) => {
                    let id = FunctionId::from_func(func, Some(contract));
                    self.functions.insert(id, func.clone());
                }
                ContractElem::Var(var) => {
                    self.state_variables
                        .insert((contract.name.clone(), var.name.clone()), var.clone());
                }
                ContractElem::Struct(s) => {
                    self.structs.insert(s.name.clone(), s.clone());
                }
                ContractElem::Enum(e) => {
                    self.enums.insert(e.name.clone(), e.clone());
                }
                ContractElem::Event(e) => {
                    self.events.insert(e.name.clone(), e.clone());
                }
                ContractElem::Error(e) => {
                    self.errors.insert(e.name.clone(), e.clone());
                }
                ContractElem::Type(t) => {
                    self.user_types.insert(t.name.clone(), t.clone());
                }
                _ => {}
            }
        }
    }

    /// Get a contract by name.
    pub fn get_contract(&self, name: &Name) -> Option<&ContractDef> {
        self.contracts.get(name)
    }

    /// Get a function by ID.
    pub fn get_function(&self, id: &FunctionId) -> Option<&FuncDef> {
        self.functions.get(id)
    }

    /// Get a state variable by contract and variable name.
    pub fn get_state_variable(&self, contract: &Name, var: &Name) -> Option<&VarDecl> {
        self.state_variables.get(&(contract.clone(), var.clone()))
    }

    /// Get all functions in a contract.
    pub fn get_contract_functions(&self, contract_name: &Name) -> Vec<&FuncDef> {
        self.functions
            .iter()
            .filter_map(|(id, func)| {
                if id.contract.as_ref() == Some(contract_name) {
                    Some(func)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all state variables in a contract.
    pub fn get_contract_state_variables(&self, contract_name: &Name) -> Vec<&VarDecl> {
        self.state_variables
            .iter()
            .filter_map(|((c, _), var)| if c == contract_name { Some(var) } else { None })
            .collect()
    }

    /// Check if a function exists.
    pub fn has_function(&self, id: &FunctionId) -> bool {
        self.functions.contains_key(id)
    }

    /// Check if a contract exists.
    pub fn has_contract(&self, name: &Name) -> bool {
        self.contracts.contains_key(name)
    }

    /// Get contract count.
    pub fn contract_count(&self) -> usize {
        self.contracts.len()
    }

    /// Get function count.
    pub fn function_count(&self) -> usize {
        self.functions.len()
    }
}

/// Pass for building the symbol table.
pub struct SymbolTablePass;

impl SymbolTablePass {
    /// Create a new symbol table pass.
    pub fn new() -> Self {
        Self
    }
}

impl Default for SymbolTablePass {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for SymbolTablePass {
    fn id(&self) -> PassId {
        PassId::SymbolTable
    }

    fn name(&self) -> &'static str {
        "Symbol Table"
    }

    fn description(&self) -> &'static str {
        "Builds a symbol table for fast lookup of program entities (contracts, functions, variables, etc.)"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Contract
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ast
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![] // No dependencies - this is a foundation pass
    }
}

impl AnalysisPass for SymbolTablePass {
    fn run(&self, context: &mut AnalysisContext) -> PassResult<()> {
        let symbol_table = SymbolTable::from_source_units(&context.source_units);
        context.store_artifact("symbol_table", symbol_table);
        context.record_ast_traversal();
        Ok(())
    }

    fn is_completed(&self, context: &AnalysisContext) -> bool {
        context.has_artifact("symbol_table")
    }
}

/// Convenience trait to get symbol table from context.
pub trait SymbolTableExt {
    fn symbol_table(&self) -> Option<&SymbolTable>;
}

impl SymbolTableExt for AnalysisContext {
    fn symbol_table(&self) -> Option<&SymbolTable> {
        self.get_artifact::<SymbolTable>("symbol_table")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table_pass() {
        let pass = SymbolTablePass::new();
        assert_eq!(pass.id(), PassId::SymbolTable);
        assert!(pass.dependencies().is_empty());
    }
}
