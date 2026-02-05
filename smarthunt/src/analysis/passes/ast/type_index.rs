//! Type Index Pass
//!
//! This pass builds an index of all types defined in the program.

use crate::analysis::pass::{Pass, AnalysisPass, PassResult};
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::analysis::context::AnalysisContext;
use solidity::ast::{
    ContractDef, ContractElem, EnumDef, Name, SourceUnit, SourceUnitElem,
    StructDef, TypeDef,
};
use std::collections::HashMap;

/// Type information.
#[derive(Debug, Clone)]
pub enum TypeInfo {
    /// Contract type.
    Contract(ContractDef),
    /// Struct type.
    Struct(StructDef),
    /// Enum type.
    Enum(EnumDef),
    /// User-defined type.
    UserDefined(TypeDef),
    /// Elementary type.
    Elementary(String),
}

impl TypeInfo {
    /// Get the name of this type.
    pub fn name(&self) -> String {
        match self {
            TypeInfo::Contract(c) => c.name.base.to_string(),
            TypeInfo::Struct(s) => s.name.base.to_string(),
            TypeInfo::Enum(e) => e.name.base.to_string(),
            TypeInfo::UserDefined(t) => t.name.base.to_string(),
            TypeInfo::Elementary(s) => s.clone(),
        }
    }
}

/// Type index for fast type lookups.
#[derive(Debug, Clone, Default)]
pub struct TypeIndex {
    /// All types indexed by name.
    pub types: HashMap<String, TypeInfo>,

    /// Contract types.
    pub contracts: HashMap<Name, ContractDef>,

    /// Struct types.
    pub structs: HashMap<Name, StructDef>,

    /// Enum types.
    pub enums: HashMap<Name, EnumDef>,

    /// User-defined types.
    pub user_types: HashMap<Name, TypeDef>,
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
                    let name = contract.name.base.to_string();
                    self.types.insert(name, TypeInfo::Contract(contract.clone()));
                    self.contracts.insert(contract.name.clone(), contract.clone());
                    self.process_contract(contract);
                }
                SourceUnitElem::Struct(s) => {
                    let name = s.name.base.to_string();
                    self.types.insert(name, TypeInfo::Struct(s.clone()));
                    self.structs.insert(s.name.clone(), s.clone());
                }
                SourceUnitElem::Enum(e) => {
                    let name = e.name.base.to_string();
                    self.types.insert(name, TypeInfo::Enum(e.clone()));
                    self.enums.insert(e.name.clone(), e.clone());
                }
                SourceUnitElem::UserType(t) => {
                    let name = t.name.base.to_string();
                    self.types.insert(name, TypeInfo::UserDefined(t.clone()));
                    self.user_types.insert(t.name.clone(), t.clone());
                }
                _ => {}
            }
        }
    }

    fn process_contract(&mut self, contract: &ContractDef) {
        for elem in &contract.body {
            match elem {
                ContractElem::Struct(s) => {
                    let name = s.name.base.to_string();
                    self.types.insert(name, TypeInfo::Struct(s.clone()));
                    self.structs.insert(s.name.clone(), s.clone());
                }
                ContractElem::Enum(e) => {
                    let name = e.name.base.to_string();
                    self.types.insert(name, TypeInfo::Enum(e.clone()));
                    self.enums.insert(e.name.clone(), e.clone());
                }
                ContractElem::Type(t) => {
                    let name = t.name.base.to_string();
                    self.types.insert(name, TypeInfo::UserDefined(t.clone()));
                    self.user_types.insert(t.name.clone(), t.clone());
                }
                _ => {}
            }
        }
    }

    /// Get a type by name.
    pub fn get_type(&self, name: &str) -> Option<&TypeInfo> {
        self.types.get(name)
    }

    /// Check if a type exists.
    pub fn has_type(&self, name: &str) -> bool {
        self.types.contains_key(name)
    }

    /// Get a contract by name.
    pub fn get_contract(&self, name: &Name) -> Option<&ContractDef> {
        self.contracts.get(name)
    }

    /// Get a struct by name.
    pub fn get_struct(&self, name: &Name) -> Option<&StructDef> {
        self.structs.get(name)
    }

    /// Get an enum by name.
    pub fn get_enum(&self, name: &Name) -> Option<&EnumDef> {
        self.enums.get(name)
    }

    /// Get type count.
    pub fn type_count(&self) -> usize {
        self.types.len()
    }
}

/// Pass for building the type index.
pub struct TypeIndexPass;

impl TypeIndexPass {
    /// Create a new type index pass.
    pub fn new() -> Self {
        Self
    }
}

impl Default for TypeIndexPass {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for TypeIndexPass {
    fn id(&self) -> PassId {
        PassId::TypeIndex
    }

    fn name(&self) -> &'static str {
        "Type Index"
    }

    fn description(&self) -> &'static str {
        "Builds an index of all types defined in the program for fast type lookups"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Contract
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ast
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![] // No dependencies
    }
}

impl AnalysisPass for TypeIndexPass {
    fn run(&self, context: &mut AnalysisContext) -> PassResult<()> {
        let type_index = TypeIndex::from_source_units(&context.source_units);
        context.store_artifact("type_index", type_index);
        context.record_ast_traversal();
        Ok(())
    }

    fn is_completed(&self, context: &AnalysisContext) -> bool {
        context.has_artifact("type_index")
    }
}

/// Convenience trait to get type index from context.
pub trait TypeIndexExt {
    fn type_index(&self) -> Option<&TypeIndex>;
}

impl TypeIndexExt for AnalysisContext {
    fn type_index(&self) -> Option<&TypeIndex> {
        self.get_artifact::<TypeIndex>("type_index")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_index_pass() {
        let pass = TypeIndexPass::new();
        assert_eq!(pass.id(), PassId::TypeIndex);
        assert!(pass.dependencies().is_empty());
    }
}
