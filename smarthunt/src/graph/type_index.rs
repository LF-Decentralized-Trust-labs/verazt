//! Type index for fast type lookups.
//!
//! This module provides fast type lookup structures.

use solidity::ast::{Name, Type, SourceUnit, SourceUnitElem, ContractElem};
use std::collections::HashMap;

/// Type index for fast type queries.
#[derive(Debug, Clone, Default)]
pub struct TypeIndex {
    /// User-defined types mapped by name
    pub user_types: HashMap<Name, Type>,
    
    /// Contract types
    pub contract_types: HashMap<Name, Type>,
    
    /// Struct types
    pub struct_types: HashMap<Name, Type>,
    
    /// Enum types
    pub enum_types: HashMap<Name, Type>,
}

impl TypeIndex {
    /// Create a new empty type index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a type index from source units.
    pub fn from_source_units(source_units: &[SourceUnit]) -> Self {
        let mut index = Self::new();
        
        for source_unit in source_units {
            index.process_source_unit(source_unit);
        }
        
        index
    }

    fn process_source_unit(&mut self, source_unit: &SourceUnit) {
        for elem in &source_unit.elems {
            match elem {
                SourceUnitElem::Contract(contract) => {
                    // Add contract as a type
                    let is_lib = matches!(contract.kind, solidity::ast::ContractKind::Library);
                    let typ = Type::Contract(solidity::ast::ContractType::new(
                        contract.name.clone(),
                        is_lib,
                        None,
                    ));
                    self.contract_types.insert(contract.name.clone(), typ);
                    
                    // Process contract body
                    for body_elem in &contract.body {
                        self.process_contract_elem(body_elem, &contract.name);
                    }
                }
                SourceUnitElem::Struct(s) => {
                    let typ = Type::Struct(solidity::ast::StructType::new(
                        s.name.clone(),
                        None,
                        solidity::ast::DataLoc::None,
                        false,
                    ));
                    self.struct_types.insert(s.name.clone(), typ);
                }
                SourceUnitElem::Enum(e) => {
                    let typ = Type::Enum(solidity::ast::EnumType::new(
                        e.name.clone(),
                        None,
                    ));
                    self.enum_types.insert(e.name.clone(), typ);
                }
                SourceUnitElem::UserType(t) => {
                    self.user_types.insert(t.name.clone(), t.base_typ.clone());
                }
                _ => {}
            }
        }
    }

    fn process_contract_elem(&mut self, elem: &ContractElem, contract_name: &Name) {
        match elem {
            ContractElem::Struct(s) => {
                let typ = Type::Struct(solidity::ast::StructType::new(
                    s.name.clone(),
                    Some(contract_name.clone()),
                    solidity::ast::DataLoc::None,
                    false,
                ));
                self.struct_types.insert(s.name.clone(), typ);
            }
            ContractElem::Enum(e) => {
                let typ = Type::Enum(solidity::ast::EnumType::new(
                    e.name.clone(),
                    Some(contract_name.clone()),
                ));
                self.enum_types.insert(e.name.clone(), typ);
            }
            ContractElem::Type(t) => {
                self.user_types.insert(t.name.clone(), t.base_typ.clone());
            }
            _ => {}
        }
    }

    /// Get a type by name.
    pub fn get_type(&self, name: &Name) -> Option<&Type> {
        self.user_types.get(name)
            .or_else(|| self.contract_types.get(name))
            .or_else(|| self.struct_types.get(name))
            .or_else(|| self.enum_types.get(name))
    }

    /// Check if a type is a contract.
    pub fn is_contract(&self, name: &Name) -> bool {
        self.contract_types.contains_key(name)
    }

    /// Check if a type is a struct.
    pub fn is_struct(&self, name: &Name) -> bool {
        self.struct_types.contains_key(name)
    }

    /// Check if a type is an enum.
    pub fn is_enum(&self, name: &Name) -> bool {
        self.enum_types.contains_key(name)
    }

    /// Check if a type is a user-defined value type.
    pub fn is_user_type(&self, name: &Name) -> bool {
        self.user_types.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_index_new() {
        let index = TypeIndex::new();
        assert!(index.user_types.is_empty());
    }
}
