//! Pass Scheduler
//!
//! Pure function: takes registered passes, returns `ExecutionSchedule`.
//! Must not mutate `AnalysisContext`.
//!
//! After step 1.8, AIR is always available (eagerly lowered), so the
//! scheduler no longer tracks a special "IR generation point".

use crate::pipeline::dependency::DependencyGraph;
use crate::pass::{Pass, PassResult};
use crate::pass::id::PassId;
use crate::pass::meta::PassRepresentation;
use std::collections::{HashMap, HashSet};

/// Execution level containing passes grouped by representation.
#[derive(Debug, Clone, Default)]
pub struct ExecutionLevel {
    /// SIR passes at this level.
    pub sir_passes: Vec<PassId>,

    /// AIR passes at this level.
    pub air_passes: Vec<PassId>,
}

impl ExecutionLevel {
    /// Check if this level is empty.
    pub fn is_empty(&self) -> bool {
        self.sir_passes.is_empty() && self.air_passes.is_empty()
    }

    /// Get total number of passes at this level.
    pub fn len(&self) -> usize {
        self.sir_passes.len() + self.air_passes.len()
    }

    /// Get all passes at this level.
    pub fn all_passes(&self) -> Vec<PassId> {
        let mut passes = Vec::with_capacity(self.len());
        passes.extend(&self.sir_passes);
        passes.extend(&self.air_passes);
        passes
    }
}

/// Schedule of passes to execute.
#[derive(Debug, Clone)]
pub struct ExecutionSchedule {
    /// Levels of passes to execute.
    pub levels: Vec<ExecutionLevel>,
}

impl ExecutionSchedule {
    /// Get total number of passes.
    pub fn total_passes(&self) -> usize {
        self.levels.iter().map(|l| l.len()).sum()
    }

    /// Check if schedule is empty.
    pub fn is_empty(&self) -> bool {
        self.levels.is_empty() || self.levels.iter().all(|l| l.is_empty())
    }
}

/// Pass scheduler for computing execution order.
pub struct PassScheduler {
    /// Dependency graph.
    dependency_graph: DependencyGraph,

    /// Pass representations.
    representations: HashMap<PassId, PassRepresentation>,

    /// Registered pass IDs.
    registered_passes: HashSet<PassId>,
}

impl Default for PassScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl PassScheduler {
    /// Create a new pass scheduler.
    pub fn new() -> Self {
        Self {
            dependency_graph: DependencyGraph::new(),
            representations: HashMap::new(),
            registered_passes: HashSet::new(),
        }
    }

    /// Register a pass with the scheduler.
    pub fn register_pass(&mut self, pass: &dyn Pass) {
        let pass_id = pass.id();

        self.registered_passes.insert(pass_id);
        self.representations.insert(pass_id, pass.representation());
        self.dependency_graph.add_pass(pass_id);

        for dep in pass.dependencies() {
            self.dependency_graph.add_dependency(pass_id, dep);
        }
    }

    /// Register multiple passes.
    pub fn register_passes(&mut self, passes: &[&dyn Pass]) {
        for pass in passes {
            self.register_pass(*pass);
        }
    }

    /// Compute the execution schedule.
    pub fn compute_schedule(&self) -> PassResult<ExecutionSchedule> {
        // Get base levels from dependency graph
        let base_levels = self.dependency_graph.compute_levels()?;

        // Reorganize levels by representation
        let mut levels: Vec<ExecutionLevel> = Vec::new();

        for pass_ids in base_levels.iter() {
            let mut level = ExecutionLevel::default();

            for &pass_id in pass_ids {
                // Categorize by representation
                match self.representations.get(&pass_id) {
                    Some(PassRepresentation::Ir) => level.sir_passes.push(pass_id),
                    Some(PassRepresentation::Air) => level.air_passes.push(pass_id),
                    _ => level.sir_passes.push(pass_id), // Default to SIR
                }
            }

            if !level.is_empty() {
                levels.push(level);
            }
        }

        Ok(ExecutionSchedule { levels })
    }

    /// Get passes that can be executed given current completion state.
    pub fn get_ready_passes(&self, completed: &HashSet<PassId>) -> Vec<PassId> {
        self.registered_passes
            .iter()
            .filter(|&pass_id| {
                !completed.contains(pass_id)
                    && self
                        .dependency_graph
                        .dependencies_satisfied(pass_id, completed)
            })
            .copied()
            .collect()
    }

    /// Get the dependency graph.
    pub fn dependency_graph(&self) -> &DependencyGraph {
        &self.dependency_graph
    }

    /// Get the number of registered passes.
    pub fn pass_count(&self) -> usize {
        self.registered_passes.len()
    }

    /// Clear all registered passes.
    pub fn clear(&mut self) {
        self.dependency_graph.clear();
        self.representations.clear();
        self.registered_passes.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pass::meta::PassLevel;

    // Mock pass for testing
    struct MockPass {
        id: PassId,
        representation: PassRepresentation,
        dependencies: Vec<PassId>,
    }

    impl Pass for MockPass {
        fn id(&self) -> PassId {
            self.id
        }
        fn name(&self) -> &'static str {
            "MockPass"
        }
        fn description(&self) -> &'static str {
            "A mock pass for testing"
        }
        fn level(&self) -> PassLevel {
            PassLevel::Contract
        }
        fn representation(&self) -> PassRepresentation {
            self.representation
        }
        fn dependencies(&self) -> Vec<PassId> {
            self.dependencies.clone()
        }
    }

    #[test]
    fn test_schedule_computation() {
        let mut scheduler = PassScheduler::new();

        let cfg = MockPass {
            id: PassId::Cfg,
            representation: PassRepresentation::Ir,
            dependencies: vec![],
        };
        let ir_cfg = MockPass {
            id: PassId::IrCfg,
            representation: PassRepresentation::Ir,
            dependencies: vec![PassId::Cfg],
        };
        let ir_cg = MockPass {
            id: PassId::IrCallGraph,
            representation: PassRepresentation::Ir,
            dependencies: vec![PassId::Cfg],
        };

        scheduler.register_pass(&cfg);
        scheduler.register_pass(&ir_cfg);
        scheduler.register_pass(&ir_cg);

        let schedule = scheduler.compute_schedule().unwrap();

        assert!(!schedule.is_empty());
    }
}
