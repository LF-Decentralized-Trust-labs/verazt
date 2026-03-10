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

use crate::pass::id::PassId;
use crate::pass::{PassError, PassResult};
use std::collections::{HashMap, HashSet};

/// Dependency graph for passes.
///
/// This structure tracks dependencies between passes and provides
/// topological sorting for execution order.
#[derive(Debug, Default)]
pub struct DependencyGraph {
    /// Edges: pass -> set of passes it depends on
    dependencies: HashMap<PassId, HashSet<PassId>>,

    /// Reverse edges: pass -> set of passes that depend on it
    dependents: HashMap<PassId, HashSet<PassId>>,

    /// All registered passes
    passes: HashSet<PassId>,
}

impl DependencyGraph {
    /// Create a new empty dependency graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a pass to the graph.
    pub fn add_pass(&mut self, pass_id: PassId) {
        self.passes.insert(pass_id);
        self.dependencies.entry(pass_id).or_default();
        self.dependents.entry(pass_id).or_default();
    }

    /// Add a dependency: `pass_id` depends on `dependency`.
    pub fn add_dependency(&mut self, pass_id: PassId, dependency: PassId) {
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
    pub fn get_dependencies(&self, pass_id: &PassId) -> Vec<PassId> {
        self.dependencies
            .get(pass_id)
            .map(|deps| deps.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get dependents of a pass (passes that depend on it).
    pub fn get_dependents(&self, pass_id: &PassId) -> Vec<PassId> {
        self.dependents
            .get(pass_id)
            .map(|deps| deps.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Check if all dependencies of a pass are satisfied.
    pub fn dependencies_satisfied(&self, pass_id: &PassId, completed: &HashSet<PassId>) -> bool {
        self.dependencies
            .get(pass_id)
            .map(|deps| deps.iter().all(|dep| completed.contains(dep)))
            .unwrap_or(true)
    }

    /// Compute topological sort of all passes.
    ///
    /// Returns passes in execution order (dependencies before dependents).
    pub fn topological_sort(&self) -> PassResult<Vec<PassId>> {
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
        pass_id: PassId,
        visited: &mut HashSet<PassId>,
        in_progress: &mut HashSet<PassId>,
        result: &mut Vec<PassId>,
    ) -> PassResult<()> {
        if visited.contains(&pass_id) {
            return Ok(());
        }

        if in_progress.contains(&pass_id) {
            return Err(PassError::CircularDependency(format!(
                "Circular dependency detected involving pass '{}'",
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
    pub fn compute_levels(&self) -> PassResult<Vec<Vec<PassId>>> {
        let sorted = self.topological_sort()?;
        let mut levels: Vec<Vec<PassId>> = Vec::new();
        let mut pass_level: HashMap<PassId, usize> = HashMap::new();

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

    #[test]
    fn test_add_dependency() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency(PassId::IrCfg, PassId::Cfg);

        assert!(graph.passes.contains(&PassId::IrCfg));
        assert!(graph.passes.contains(&PassId::Cfg));
    }

    #[test]
    fn test_topological_sort() {
        let mut graph = DependencyGraph::new();
        graph.add_pass(PassId::Cfg);
        graph.add_dependency(PassId::IrCfg, PassId::Cfg);
        graph.add_dependency(PassId::IrCallGraph, PassId::Cfg);

        let sorted = graph.topological_sort().unwrap();

        let cfg_pos = sorted.iter().position(|&p| p == PassId::Cfg).unwrap();
        let ir_cfg_pos = sorted.iter().position(|&p| p == PassId::IrCfg).unwrap();
        let ir_cg_pos = sorted
            .iter()
            .position(|&p| p == PassId::IrCallGraph)
            .unwrap();

        assert!(cfg_pos < ir_cfg_pos);
        assert!(cfg_pos < ir_cg_pos);
    }

    #[test]
    fn test_compute_levels() {
        let mut graph = DependencyGraph::new();
        graph.add_pass(PassId::Cfg);
        graph.add_pass(PassId::DataFlow);
        graph.add_dependency(PassId::IrCfg, PassId::Cfg);
        graph.add_dependency(PassId::IrCallGraph, PassId::Cfg);
        graph.add_dependency(PassId::TaintAnalysis, PassId::IrCallGraph);

        let levels = graph.compute_levels().unwrap();

        assert_eq!(levels.len(), 3);
        assert!(levels[0].contains(&PassId::Cfg));
        assert!(levels[0].contains(&PassId::DataFlow));
    }

    #[test]
    fn test_circular_dependency() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency(PassId::Cfg, PassId::IrCfg);
        graph.add_dependency(PassId::IrCfg, PassId::Cfg);

        let result = graph.topological_sort();
        assert!(result.is_err());
    }
}
