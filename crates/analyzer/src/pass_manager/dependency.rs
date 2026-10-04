//! Dependency Graph for Pass Scheduling
//!
//! This module provides a dependency graph implementation for
//! computing pass execution order.
//!
//! ## Design note (Step 3.2 evaluation)
//!
//! Replacing this hand-rolled graph with `petgraph` was considered but
//! rejected: the current implementation is small, fully tested, and
//! exposes exactly the API needed by the scheduler (topological sort,
//! level computation, cycle detection).  `petgraph` would add wrapping
//! overhead without meaningfully reducing code.  CFA algorithms that
//! *do* benefit from `petgraph` (dominator trees, SCC) live in
//! `frameworks::cfa`.

use crate::passes::base::{PassError, PassResult};
use std::any::TypeId;
use std::collections::{HashMap, HashSet};

// ========================================================================
// Data Structures
// ========================================================================

/// Dependency graph for passes.
///
/// Orders passes so that each runs after the passes it depends on. Pass
/// names break ties, so the order is deterministic, and name the passes in
/// errors.
#[derive(Debug, Default)]
pub struct DependencyGraph {
    /// Each pass, with the passes it depends on.
    dependencies: HashMap<TypeId, Vec<TypeId>>,

    /// The name of each pass.
    names: HashMap<TypeId, &'static str>,
}

// ========================================================================
// DependencyGraph Implementations
// ========================================================================

impl DependencyGraph {
    /// Create a new empty dependency graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add pass `pass_id` named `name`, which depends on `dependencies`.
    pub fn add_pass(&mut self, pass_id: TypeId, name: &'static str, dependencies: Vec<TypeId>) {
        self.names.insert(pass_id, name);
        self.dependencies.insert(pass_id, dependencies);
    }

    /// Compute topological sort of all passes.
    ///
    /// Returns passes in execution order (dependencies before dependents),
    /// visiting independent passes in name order.
    pub fn topological_sort(&self) -> PassResult<Vec<TypeId>> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut path = Vec::new();

        for pass_id in self.sorted_by_name(self.names.keys().copied()) {
            self.visit(pass_id, &mut visited, &mut path, &mut result)?;
        }

        Ok(result)
    }

    /// Compute execution levels for parallel execution.
    ///
    /// Returns a vector of levels, each sorted by pass name, where the
    /// passes of a level depend only on passes of earlier levels and can
    /// therefore run in parallel.
    pub fn compute_levels(&self) -> PassResult<Vec<Vec<TypeId>>> {
        let mut levels: Vec<Vec<TypeId>> = Vec::new();
        let mut pass_level: HashMap<TypeId, usize> = HashMap::new();

        for pass_id in self.topological_sort()? {
            let level = self.dependencies[&pass_id]
                .iter()
                .map(|dep| pass_level[dep] + 1)
                .max()
                .unwrap_or(0);
            pass_level.insert(pass_id, level);
            if levels.len() <= level {
                levels.resize_with(level + 1, Vec::new);
            }
            levels[level].push(pass_id);
        }

        Ok(levels.into_iter().map(|level| self.sorted_by_name(level)).collect())
    }

    /// Depth-first visit of `pass_id`, appending it to `result` after its
    /// dependencies. `path` holds the passes being visited, to report a
    /// cycle through them.
    fn visit(
        &self,
        pass_id: TypeId,
        visited: &mut HashSet<TypeId>,
        path: &mut Vec<TypeId>,
        result: &mut Vec<TypeId>,
    ) -> PassResult<()> {
        if visited.contains(&pass_id) {
            return Ok(());
        }

        if let Some(start) = path.iter().position(|&p| p == pass_id) {
            let cycle: Vec<&str> =
                path[start..].iter().chain([&pass_id]).map(|&p| self.name(p)).collect();
            return Err(PassError::CircularDependency(cycle.join(" -> ")));
        }

        let dependencies = &self.dependencies[&pass_id];
        if dependencies.iter().any(|dep| !self.names.contains_key(dep)) {
            return Err(PassError::UnregisteredDependency(self.name(pass_id).to_string()));
        }

        path.push(pass_id);
        for dep in self.sorted_by_name(dependencies.iter().copied()) {
            self.visit(dep, visited, path, result)?;
        }
        path.pop();

        visited.insert(pass_id);
        result.push(pass_id);

        Ok(())
    }

    /// The name of pass `pass_id`, which must be in the graph.
    fn name(&self, pass_id: TypeId) -> &'static str {
        self.names[&pass_id]
    }

    /// `pass_ids`, which must be in the graph, sorted by pass name.
    fn sorted_by_name(&self, pass_ids: impl IntoIterator<Item = TypeId>) -> Vec<TypeId> {
        let mut sorted: Vec<TypeId> = pass_ids.into_iter().collect();
        sorted.sort_by_key(|&p| self.name(p));
        sorted
    }
}

// ========================================================================
// Tests
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Marker types to get distinct TypeIds for testing
    struct PassA;
    struct PassB;
    struct PassC;
    struct PassD;
    struct PassE;

    fn id<T: 'static>() -> TypeId {
        TypeId::of::<T>()
    }

    #[test]
    fn test_topological_sort() {
        let mut graph = DependencyGraph::new();
        graph.add_pass(id::<PassC>(), "c", vec![id::<PassA>()]);
        graph.add_pass(id::<PassB>(), "b", vec![id::<PassA>()]);
        graph.add_pass(id::<PassA>(), "a", vec![]);

        let sorted = graph.topological_sort().unwrap();

        assert_eq!(sorted, [id::<PassA>(), id::<PassB>(), id::<PassC>()]);
    }

    #[test]
    fn test_compute_levels() {
        let mut graph = DependencyGraph::new();
        graph.add_pass(id::<PassE>(), "e", vec![id::<PassC>()]);
        graph.add_pass(id::<PassD>(), "d", vec![]);
        graph.add_pass(id::<PassC>(), "c", vec![id::<PassA>()]);
        graph.add_pass(id::<PassB>(), "b", vec![id::<PassA>()]);
        graph.add_pass(id::<PassA>(), "a", vec![]);

        let levels = graph.compute_levels().unwrap();

        assert_eq!(
            levels,
            [
                vec![id::<PassA>(), id::<PassD>()],
                vec![id::<PassB>(), id::<PassC>()],
                vec![id::<PassE>()],
            ]
        );
    }

    #[test]
    fn test_circular_dependency_names_the_cycle() {
        let mut graph = DependencyGraph::new();
        graph.add_pass(id::<PassA>(), "a", vec![id::<PassB>()]);
        graph.add_pass(id::<PassB>(), "b", vec![id::<PassA>()]);

        let err = graph.topological_sort().unwrap_err();

        assert!(err.to_string().contains("a -> b -> a"), "{err}");
    }

    #[test]
    fn test_unregistered_dependency_names_the_dependent() {
        let mut graph = DependencyGraph::new();
        graph.add_pass(id::<PassB>(), "b", vec![id::<PassA>()]);

        let err = graph.topological_sort().unwrap_err();

        assert!(matches!(&err, PassError::UnregisteredDependency(name) if name == "b"), "{err}");
    }
}
