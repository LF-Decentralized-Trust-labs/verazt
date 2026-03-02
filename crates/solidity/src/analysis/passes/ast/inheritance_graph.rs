//! Inheritance Graph Pass
//!
//! This pass builds the inheritance graph for contracts.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::{AnalysisPass, Pass, PassResult};
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::ast::{ContractDef, Name, SourceUnit, SourceUnitElem};
use std::collections::{HashMap, HashSet};

/// Inheritance graph for contracts.
#[derive(Debug, Clone, Default)]
pub struct InheritanceGraph {
    /// Direct parents of each contract.
    pub parents: HashMap<Name, Vec<Name>>,

    /// Direct children of each contract.
    pub children: HashMap<Name, Vec<Name>>,

    /// All ancestors (transitive closure of parents).
    pub ancestors: HashMap<Name, HashSet<Name>>,

    /// All descendants (transitive closure of children).
    pub descendants: HashMap<Name, HashSet<Name>>,

    /// Linearized inheritance order (C3 linearization).
    pub linearization: HashMap<Name, Vec<Name>>,
}

impl InheritanceGraph {
    /// Create a new empty inheritance graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build inheritance graph from source units.
    pub fn from_source_units(source_units: &[SourceUnit]) -> Self {
        let mut graph = Self::new();

        // First pass: collect all contracts and their direct parents
        for source_unit in source_units {
            for elem in &source_unit.elems {
                if let SourceUnitElem::Contract(contract) = elem {
                    graph.add_contract(contract);
                }
            }
        }

        // Second pass: compute transitive closures
        graph.compute_ancestors();
        graph.compute_descendants();

        graph
    }

    fn add_contract(&mut self, contract: &ContractDef) {
        let name = contract.name.clone();

        // Extract parent names from base contracts
        let parent_names: Vec<Name> = contract
            .base_contracts
            .iter()
            .map(|bc| bc.name.clone())
            .collect();

        // Add to parents map
        self.parents.insert(name.clone(), parent_names.clone());

        // Add to children map for each parent
        for parent in &parent_names {
            self.children
                .entry(parent.clone())
                .or_default()
                .push(name.clone());
        }
    }

    fn compute_ancestors(&mut self) {
        let contracts: Vec<Name> = self.parents.keys().cloned().collect();

        for contract in &contracts {
            let mut ancestors = HashSet::new();
            self.collect_ancestors(contract, &mut ancestors);
            self.ancestors.insert(contract.clone(), ancestors);
        }
    }

    fn collect_ancestors(&self, contract: &Name, ancestors: &mut HashSet<Name>) {
        if let Some(parents) = self.parents.get(contract) {
            for parent in parents {
                if ancestors.insert(parent.clone()) {
                    self.collect_ancestors(parent, ancestors);
                }
            }
        }
    }

    fn compute_descendants(&mut self) {
        let contracts: Vec<Name> = self.parents.keys().cloned().collect();

        for contract in &contracts {
            let mut descendants = HashSet::new();
            self.collect_descendants(contract, &mut descendants);
            self.descendants.insert(contract.clone(), descendants);
        }
    }

    fn collect_descendants(&self, contract: &Name, descendants: &mut HashSet<Name>) {
        if let Some(children) = self.children.get(contract) {
            for child in children {
                if descendants.insert(child.clone()) {
                    self.collect_descendants(child, descendants);
                }
            }
        }
    }

    /// Get direct parents of a contract.
    pub fn get_parents(&self, contract: &Name) -> Vec<Name> {
        self.parents.get(contract).cloned().unwrap_or_default()
    }

    /// Get direct children of a contract.
    pub fn get_children(&self, contract: &Name) -> Vec<Name> {
        self.children.get(contract).cloned().unwrap_or_default()
    }

    /// Get all ancestors of a contract.
    pub fn get_ancestors(&self, contract: &Name) -> HashSet<Name> {
        self.ancestors.get(contract).cloned().unwrap_or_default()
    }

    /// Get all descendants of a contract.
    pub fn get_descendants(&self, contract: &Name) -> HashSet<Name> {
        self.descendants.get(contract).cloned().unwrap_or_default()
    }

    /// Check if one contract inherits from another.
    pub fn inherits_from(&self, child: &Name, parent: &Name) -> bool {
        self.ancestors
            .get(child)
            .map(|a| a.contains(parent))
            .unwrap_or(false)
    }

    /// Check if one contract is an ancestor of another.
    pub fn is_ancestor_of(&self, ancestor: &Name, descendant: &Name) -> bool {
        self.inherits_from(descendant, ancestor)
    }

    /// Get the inheritance depth of a contract.
    pub fn depth(&self, contract: &Name) -> usize {
        self.ancestors.get(contract).map(|a| a.len()).unwrap_or(0)
    }

    /// Check if a contract is a leaf (has no children).
    pub fn is_leaf(&self, contract: &Name) -> bool {
        self.children
            .get(contract)
            .map(|c| c.is_empty())
            .unwrap_or(true)
    }

    /// Check if a contract is a root (has no parents).
    pub fn is_root(&self, contract: &Name) -> bool {
        self.parents
            .get(contract)
            .map(|p| p.is_empty())
            .unwrap_or(true)
    }
}

/// Pass for building the inheritance graph.
pub struct InheritanceGraphPass;

impl InheritanceGraphPass {
    /// Create a new inheritance graph pass.
    pub fn new() -> Self {
        Self
    }
}

impl Default for InheritanceGraphPass {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for InheritanceGraphPass {
    fn id(&self) -> PassId {
        PassId::InheritanceGraph
    }

    fn name(&self) -> &'static str {
        "Inheritance Graph"
    }

    fn description(&self) -> &'static str {
        "Builds the inheritance graph for contracts, including transitive ancestor/descendant relationships"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Contract
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ast
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::SymbolTable]
    }
}

impl AnalysisPass for InheritanceGraphPass {
    fn run(&self, context: &mut AnalysisContext) -> PassResult<()> {
        let graph = InheritanceGraph::from_source_units(&context.source_units);
        context.store_artifact("inheritance_graph", graph);
        context.record_ast_traversal();
        Ok(())
    }

    fn is_completed(&self, context: &AnalysisContext) -> bool {
        context.has_artifact("inheritance_graph")
    }
}

/// Convenience trait to get inheritance graph from context.
pub trait InheritanceGraphExt {
    fn inheritance_graph(&self) -> Option<&InheritanceGraph>;
}

impl InheritanceGraphExt for AnalysisContext {
    fn inheritance_graph(&self) -> Option<&InheritanceGraph> {
        self.get_artifact::<InheritanceGraph>("inheritance_graph")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inheritance_graph_pass() {
        let pass = InheritanceGraphPass::new();
        assert_eq!(pass.id(), PassId::InheritanceGraph);
        assert_eq!(pass.dependencies(), vec![PassId::SymbolTable]);
    }
}
