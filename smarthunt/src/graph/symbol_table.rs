//! Symbol table for SmartHunt.
//!
//! This module provides data structures for fast lookup of program entities.

use solidity::ast::{
    ContractDef, EnumDef, ErrorDef, EventDef, FuncDef, Name, SourceUnit, SourceUnitElem,
    StructDef, TypeDef, VarDecl, ContractElem, FuncKind,
};
use std::collections::HashMap;

/// Unique identifier for a function.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionId {
    /// Contract name (None for free functions)
    pub contract: Option<Name>,
    /// Function name
    pub name: Name,
    /// Function kind
    pub kind: FuncKind,
}

impl FunctionId {
    pub fn new(contract: Option<Name>, name: Name, kind: FuncKind) -> Self {
        Self { contract, name, kind }
    }

    pub fn from_func(func: &FuncDef, contract: Option<&ContractDef>) -> Self {
        Self {
            contract: contract.map(|c| c.name.clone()),
            name: func.name.clone(),
            kind: func.kind.clone(),
        }
    }
}

/// Unique identifier for a contract.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContractId {
    pub name: Name,
}

impl ContractId {
    pub fn new(name: Name) -> Self {
        Self { name }
    }

    pub fn from_contract(contract: &ContractDef) -> Self {
        Self { name: contract.name.clone() }
    }
}

/// Unique identifier for a variable.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VariableId {
    /// Contract name (None for free variables)
    pub contract: Option<Name>,
    /// Function name (None for state variables)
    pub function: Option<Name>,
    /// Variable name
    pub name: Name,
    /// Is this a state variable?
    pub is_state_var: bool,
}

impl VariableId {
    pub fn new(contract: Option<Name>, function: Option<Name>, name: Name, is_state_var: bool) -> Self {
        Self { contract, function, name, is_state_var }
    }
}

/// Symbol kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    Contract,
    Function,
    Variable,
    Struct,
    Enum,
    Event,
    Error,
    UserType,
}

/// A symbol in the symbol table.
#[derive(Debug, Clone)]
pub enum Symbol {
    Contract(ContractDef),
    Function(FuncDef),
    Variable(VarDecl),
    Struct(StructDef),
    Enum(EnumDef),
    Event(EventDef),
    Error(ErrorDef),
    UserType(TypeDef),
}

impl Symbol {
    pub fn kind(&self) -> SymbolKind {
        match self {
            Symbol::Contract(_) => SymbolKind::Contract,
            Symbol::Function(_) => SymbolKind::Function,
            Symbol::Variable(_) => SymbolKind::Variable,
            Symbol::Struct(_) => SymbolKind::Struct,
            Symbol::Enum(_) => SymbolKind::Enum,
            Symbol::Event(_) => SymbolKind::Event,
            Symbol::Error(_) => SymbolKind::Error,
            Symbol::UserType(_) => SymbolKind::UserType,
        }
    }

    pub fn name(&self) -> Name {
        match self {
            Symbol::Contract(c) => c.name.clone(),
            Symbol::Function(f) => f.name.clone(),
            Symbol::Variable(v) => v.name.clone(),
            Symbol::Struct(s) => s.name.clone(),
            Symbol::Enum(e) => e.name.clone(),
            Symbol::Event(e) => e.name.clone(),
            Symbol::Error(e) => e.name.clone(),
            Symbol::UserType(t) => t.name.clone(),
        }
    }
}

/// Symbol table for fast lookup of program entities.
#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    /// All contracts indexed by name
    pub contracts: HashMap<Name, ContractDef>,
    
    /// All functions indexed by FunctionId
    pub functions: HashMap<FunctionId, FuncDef>,
    
    /// All state variables indexed by (contract name, var name)
    pub state_variables: HashMap<(Name, Name), VarDecl>,
    
    /// All structs indexed by name
    pub structs: HashMap<Name, StructDef>,
    
    /// All enums indexed by name
    pub enums: HashMap<Name, EnumDef>,
    
    /// All events indexed by name
    pub events: HashMap<Name, EventDef>,
    
    /// All errors indexed by name
    pub errors: HashMap<Name, ErrorDef>,
    
    /// All user-defined types indexed by name
    pub user_types: HashMap<Name, TypeDef>,
    
    /// Free functions (not in a contract)
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
                    self.contracts.insert(contract.name.clone(), contract.clone());
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
                    self.state_variables.insert(
                        (contract.name.clone(), var.name.clone()),
                        var.clone(),
                    );
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
            .filter_map(|((c, _), var)| {
                if c == contract_name {
                    Some(var)
                } else {
                    None
                }
            })
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table_new() {
        let table = SymbolTable::new();
        assert!(table.contracts.is_empty());
        assert!(table.functions.is_empty());
    }
}
