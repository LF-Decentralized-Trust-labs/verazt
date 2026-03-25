//! Dependency Graph for Pass Scheduling
//!
//! This module provides a dependency graph implementation for
//! computing pass execution order.
//!
//! ## Design note (Step 3.2 evaluation)
//!
//! Replacing this hand-rolled graph with `petgraph` was considered but
//! rejected: the current implementation is ~170 lines, fully tested, and
//! exposes exactly the API needed by the scheduler (topological sort,
//! level computation, cycle detection).  `petgraph` would add wrapping
//! overhead without meaningfully reducing code.  CFA algorithms that
//! *do* benefit from `petgraph` (dominator trees, SCC) live in
//! `frameworks::cfa`.

use crate::analysis::passes::base::{PassError, PassResult};
use std::any::TypeId;
use std::collections::{HashMap, HashSet};

/// Dependency graph for passes.
///
/// This structure tracks dependencies between passes and provides
/// topological sorting for execution order.
#[derive(Debug, Default)]
pub struct DependencyGraph {
    /// Edges: pass -> set of passes it depends on
    dependencies: HashMap<TypeId, HashSet<TypeId>>,

    /// Reverse edges: pass -> set of passes that depend on it
    dependents: HashMap<TypeId, HashSet<TypeId>>,

    /// All registered passes
    passes: HashSet<TypeId>,
}

impl DependencyGraph {
    /// Create a new empty dependency graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a pass to the graph.
    pub fn add_pass(&mut self, pass_id: TypeId) {
        self.passes.insert(pass_id);
        self.dependencies.entry(pass_id).or_default();
        self.dependents.entry(pass_id).or_default();
    }

    /// Add a dependency: `pass_id` depends on `dependency`.
    pub fn add_dependency(&mut self, pass_id: TypeId, dependency: TypeId) {
        self.add_pass(pass_id);
        self.add_pass(dependency);

        self.dependencies
            .entry(pass_id)
            .or_default()
            .insert(dependency);

        self.dependents
            .entry(dependency)
            .or_default()
            .insert(pass_id);
    }

    /// Get dependencies of a pass.
    pub fn get_dependencies(&self, pass_id: &TypeId) -> Vec<TypeId> {
        self.dependencies
            .get(pass_id)
            .map(|deps| deps.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get dependents of a pass (passes that depend on it).
    pub fn get_dependents(&self, pass_id: &TypeId) -> Vec<TypeId> {
        self.dependents
            .get(pass_id)
            .map(|deps| deps.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Check if all dependencies of a pass are satisfied.
    pub fn dependencies_satisfied(&self, pass_id: &TypeId, completed: &HashSet<TypeId>) -> bool {
        self.dependencies
            .get(pass_id)
            .map(|deps| deps.iter().all(|dep| completed.contains(dep)))
            .unwrap_or(true)
    }

    /// Compute topological sort of all passes.
    ///
    /// Returns passes in execution order (dependencies before dependents).
    pub fn topological_sort(&self) -> PassResult<Vec<TypeId>> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut in_progress = HashSet::new();

        for pass_id in &self.passes {
            self.visit(*pass_id, &mut visited, &mut in_progress, &mut result)?;
        }

        Ok(result)
    }

    fn visit(
        &self,
        pass_id: TypeId,
        visited: &mut HashSet<TypeId>,
        in_progress: &mut HashSet<TypeId>,
        result: &mut Vec<TypeId>,
    ) -> PassResult<()> {
        if visited.contains(&pass_id) {
            return Ok(());
        }

        if in_progress.contains(&pass_id) {
            return Err(PassError::CircularDependency(format!(
                "Circular dependency detected involving pass '{:?}'",
                pass_id
            )));
        }

        in_progress.insert(pass_id);

        if let Some(deps) = self.dependencies.get(&pass_id) {
            for dep in deps {
                if self.passes.contains(dep) {
                    self.visit(*dep, visited, in_progress, result)?;
                }
            }
        }

        in_progress.remove(&pass_id);
        visited.insert(pass_id);
        result.push(pass_id);

        Ok(())
    }

    /// Compute execution levels for parallel execution.
    ///
    /// Returns a vector of levels, where each level contains passes
    /// that can be executed in parallel.
    pub fn compute_levels(&self) -> PassResult<Vec<Vec<TypeId>>> {
        let sorted = self.topological_sort()?;
        let mut levels: Vec<Vec<TypeId>> = Vec::new();
        let mut pass_level: HashMap<TypeId, usize> = HashMap::new();

        for pass_id in sorted {
            // Compute level based on dependencies
            let level = self
                .dependencies
                .get(&pass_id)
                .map(|deps| {
                    deps.iter()
                        .filter_map(|dep| pass_level.get(dep))
                        .max()
                        .map(|l| l + 1)
                        .unwrap_or(0)
                })
                .unwrap_or(0);

            pass_level.insert(pass_id, level);

            // Ensure we have enough levels
            while levels.len() <= level {
                levels.push(Vec::new());
            }

            levels[level].push(pass_id);
        }

        Ok(levels)
    }

    /// Get the number of passes in the graph.
    pub fn len(&self) -> usize {
        self.passes.len()
    }

    /// Check if the graph is empty.
    pub fn is_empty(&self) -> bool {
        self.passes.is_empty()
    }

    /// Clear the graph.
    pub fn clear(&mut self) {
        self.dependencies.clear();
        self.dependents.clear();
        self.passes.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Marker types to get distinct TypeIds for testing
    struct PassA; // analogous to "Cfg"
    struct PassB; // analogous to "IrCfg"
    struct PassC; // analogous to "IrCallGraph"
    struct PassD; // analogous to "DataFlow"
    struct PassE; // analogous to "TaintAnalysis"

    #[test]
    fn test_add_dependency() {
        let mut graph = DependencyGraph::new();
        let id_a = TypeId::of::<PassA>();
        let id_b = TypeId::of::<PassB>();

        graph.add_dependency(id_b, id_a);

        assert!(graph.passes.contains(&id_b));
        assert!(graph.passes.contains(&id_a));
    }

    #[test]
    fn test_topological_sort() {
        let mut graph = DependencyGraph::new();
        let id_a = TypeId::of::<PassA>();
        let id_b = TypeId::of::<PassB>();
        let id_c = TypeId::of::<PassC>();

        graph.add_pass(id_a);
        graph.add_dependency(id_b, id_a);
        graph.add_dependency(id_c, id_a);

        let sorted = graph.topological_sort().unwrap();

        let a_pos = sorted.iter().position(|&p| p == id_a).unwrap();
        let b_pos = sorted.iter().position(|&p| p == id_b).unwrap();
        let c_pos = sorted.iter().position(|&p| p == id_c).unwrap();

        assert!(a_pos < b_pos);
        assert!(a_pos < c_pos);
    }

    #[test]
    fn test_compute_levels() {
        let mut graph = DependencyGraph::new();
        let id_a = TypeId::of::<PassA>();
        let id_b = TypeId::of::<PassB>();
        let id_c = TypeId::of::<PassC>();
        let id_d = TypeId::of::<PassD>();
        let id_e = TypeId::of::<PassE>();

        graph.add_pass(id_a);
        graph.add_pass(id_d);
        graph.add_dependency(id_b, id_a);
        graph.add_dependency(id_c, id_a);
        graph.add_dependency(id_e, id_c);

        let levels = graph.compute_levels().unwrap();

        assert_eq!(levels.len(), 3);
        assert!(levels[0].contains(&id_a));
        assert!(levels[0].contains(&id_d));
    }

    #[test]
    fn test_circular_dependency() {
        let mut graph = DependencyGraph::new();
        let id_a = TypeId::of::<PassA>();
        let id_b = TypeId::of::<PassB>();

        graph.add_dependency(id_a, id_b);
        graph.add_dependency(id_b, id_a);

        let result = graph.topological_sort();
        assert!(result.is_err());
    }
}
