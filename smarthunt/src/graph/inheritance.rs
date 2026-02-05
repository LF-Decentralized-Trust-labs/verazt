//! Inheritance graph structures.
//!
//! This module provides data structures for representing contract inheritance.

use solidity::ast::{ContractDef, Name, SourceUnit, SourceUnitElem, ContractKind};
use std::collections::{HashMap, HashSet};

/// Inheritance relationship.
#[derive(Debug, Clone)]
pub struct InheritanceEdge {
    /// Child contract
    pub child: Name,
    /// Parent contract
    pub parent: Name,
    /// Arguments to parent constructor
    pub has_constructor_args: bool,
}

/// Inheritance graph for contracts.
#[derive(Debug, Clone, Default)]
pub struct InheritanceGraph {
    /// Contract name -> list of direct parents
    pub parents: HashMap<Name, Vec<Name>>,
    
    /// Contract name -> list of direct children
    pub children: HashMap<Name, Vec<Name>>,
    
    /// All edges
    pub edges: Vec<InheritanceEdge>,
    
    /// C3 linearization order for each contract
    pub linearization: HashMap<Name, Vec<Name>>,
    
    /// Contract kinds
    pub contract_kinds: HashMap<Name, ContractKind>,
}

impl InheritanceGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build inheritance graph from source units.
    pub fn from_source_units(source_units: &[SourceUnit]) -> Self {
        let mut graph = Self::new();
        
        for source_unit in source_units {
            for elem in &source_unit.elems {
                if let SourceUnitElem::Contract(contract) = elem {
                    graph.add_contract(contract);
                }
            }
        }
        
        // Compute linearization for all contracts
        let contracts: Vec<Name> = graph.contract_kinds.keys().cloned().collect();
        for contract in contracts {
            graph.compute_linearization(&contract);
        }
        
        graph
    }

    fn add_contract(&mut self, contract: &ContractDef) {
        let name = contract.name.clone();
        self.contract_kinds.insert(name.clone(), contract.kind.clone());
        
        let parent_names: Vec<Name> = contract.base_contracts
            .iter()
            .map(|base| base.name.clone())
            .collect();
        
        for base in &contract.base_contracts {
            let edge = InheritanceEdge {
                child: name.clone(),
                parent: base.name.clone(),
                has_constructor_args: !base.args.is_empty(),
            };
            self.edges.push(edge);
            
            self.children
                .entry(base.name.clone())
                .or_default()
                .push(name.clone());
        }
        
        self.parents.insert(name, parent_names);
    }

    fn compute_linearization(&mut self, contract: &Name) {
        if self.linearization.contains_key(contract) {
            return;
        }
        
        let mut result = vec![contract.clone()];
        
        if let Some(parents) = self.parents.get(contract).cloned() {
            // Compute linearization for all parents first
            for parent in &parents {
                self.compute_linearization(parent);
            }
            
            // Merge parent linearizations using C3 algorithm
            let parent_linearizations: Vec<Vec<Name>> = parents
                .iter()
                .filter_map(|p| self.linearization.get(p).cloned())
                .collect();
            
            result.extend(self.c3_merge(parent_linearizations, &parents));
        }
        
        self.linearization.insert(contract.clone(), result);
    }

    fn c3_merge(&self, mut lists: Vec<Vec<Name>>, parents: &[Name]) -> Vec<Name> {
        // Add parents list as last list to merge
        lists.push(parents.to_vec());
        
        let mut result = Vec::new();
        
        while !lists.iter().all(|l| l.is_empty()) {
            // Find a good head
            let mut found = None;
            
            for list in &lists {
                if list.is_empty() {
                    continue;
                }
                let head = &list[0];
                
                // Check if head appears in the tail of any list
                let in_tail = lists.iter().any(|l| {
                    l.len() > 1 && l[1..].contains(head)
                });
                
                if !in_tail {
                    found = Some(head.clone());
                    break;
                }
            }
            
            let Some(good_head) = found else {
                // No good head found - inconsistent hierarchy
                break;
            };
            
            result.push(good_head.clone());
            
            // Remove good_head from all lists
            for list in &mut lists {
                list.retain(|x| x != &good_head);
            }
        }
        
        result
    }

    /// Get the direct parents of a contract.
    pub fn get_parents(&self, contract: &Name) -> Vec<Name> {
        self.parents.get(contract).cloned().unwrap_or_default()
    }

    /// Get the direct children of a contract.
    pub fn get_children(&self, contract: &Name) -> Vec<Name> {
        self.children.get(contract).cloned().unwrap_or_default()
    }

    /// Get all ancestors (transitive parents) of a contract.
    pub fn get_ancestors(&self, contract: &Name) -> HashSet<Name> {
        let mut result = HashSet::new();
        self.collect_ancestors(contract, &mut result);
        result
    }

    fn collect_ancestors(&self, contract: &Name, result: &mut HashSet<Name>) {
        for parent in self.get_parents(contract) {
            if result.insert(parent.clone()) {
                self.collect_ancestors(&parent, result);
            }
        }
    }

    /// Get the C3 linearization order for a contract.
    pub fn get_linearization(&self, contract: &Name) -> Vec<Name> {
        self.linearization.get(contract).cloned().unwrap_or_default()
    }

    /// Check if contract A inherits from contract B (directly or transitively).
    pub fn inherits_from(&self, child: &Name, parent: &Name) -> bool {
        if child == parent {
            return false;
        }
        self.get_ancestors(child).contains(parent)
    }

    /// Check if a contract is an interface.
    pub fn is_interface(&self, contract: &Name) -> bool {
        matches!(self.contract_kinds.get(contract), Some(ContractKind::Interface))
    }

    /// Check if a contract is a library.
    pub fn is_library(&self, contract: &Name) -> bool {
        matches!(self.contract_kinds.get(contract), Some(ContractKind::Library))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inheritance_graph_new() {
        let graph = InheritanceGraph::new();
        assert!(graph.parents.is_empty());
        assert!(graph.children.is_empty());
    }
}
