//! Pass Scheduler
//!
//! This module provides the pass scheduler that computes execution order
//! based on dependencies and representation requirements.

use crate::analysis::dependency::DependencyGraph;
use crate::analysis::pass::{Pass, PassResult};
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_representation::PassRepresentation;
use std::collections::{HashMap, HashSet};

/// Execution level containing passes grouped by representation.
#[derive(Debug, Clone, Default)]
pub struct ExecutionLevel {
    /// AST passes at this level.
    pub ast_passes: Vec<PassId>,

    /// IR passes at this level.
    pub ir_passes: Vec<PassId>,

    /// Hybrid passes at this level.
    pub hybrid_passes: Vec<PassId>,
}

impl ExecutionLevel {
    /// Check if this level is empty.
    pub fn is_empty(&self) -> bool {
        self.ast_passes.is_empty() && self.ir_passes.is_empty() && self.hybrid_passes.is_empty()
    }

    /// Get total number of passes at this level.
    pub fn len(&self) -> usize {
        self.ast_passes.len() + self.ir_passes.len() + self.hybrid_passes.len()
    }

    /// Get all passes at this level.
    pub fn all_passes(&self) -> Vec<PassId> {
        let mut passes = Vec::with_capacity(self.len());
        passes.extend(&self.ast_passes);
        passes.extend(&self.ir_passes);
        passes.extend(&self.hybrid_passes);
        passes
    }
}

/// Schedule of passes to execute.
#[derive(Debug, Clone)]
pub struct ExecutionSchedule {
    /// Levels of passes to execute.
    pub levels: Vec<ExecutionLevel>,

    /// Whether IR generation is needed.
    pub needs_ir: bool,

    /// Level at which IR generation occurs (if needed).
    pub ir_generation_level: Option<usize>,
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

    /// Check if any registered pass requires IR.
    pub fn needs_ir(&self) -> bool {
        self.representations.values().any(|rep| rep.requires_ir())
    }

    /// Compute the execution schedule.
    pub fn compute_schedule(&self) -> PassResult<ExecutionSchedule> {
        // Get base levels from dependency graph
        let base_levels = self.dependency_graph.compute_levels()?;

        // Check if IR is needed
        let needs_ir = self.needs_ir();

        // Reorganize levels by representation
        let mut levels: Vec<ExecutionLevel> = Vec::new();
        let mut ir_generation_level = None;

        for pass_ids in base_levels.iter() {
            let mut level = ExecutionLevel::default();

            for &pass_id in pass_ids {
                // Check if this is IR generation
                if pass_id == PassId::IrGeneration {
                    ir_generation_level = Some(levels.len());
                }

                // Categorize by representation
                match self.representations.get(&pass_id) {
                    Some(PassRepresentation::Ast) => level.ast_passes.push(pass_id),
                    Some(PassRepresentation::Ir) => level.ir_passes.push(pass_id),
                    Some(PassRepresentation::Hybrid) => level.hybrid_passes.push(pass_id),
                    None => level.ast_passes.push(pass_id), // Default to AST
                }
            }

            if !level.is_empty() {
                levels.push(level);
            }
        }

        Ok(ExecutionSchedule { levels, needs_ir, ir_generation_level })
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
    use crate::analysis::pass_level::PassLevel;

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

        let symbol_table = MockPass {
            id: PassId::SymbolTable,
            representation: PassRepresentation::Ast,
            dependencies: vec![],
        };
        let type_index = MockPass {
            id: PassId::TypeIndex,
            representation: PassRepresentation::Ast,
            dependencies: vec![PassId::SymbolTable],
        };
        let call_graph = MockPass {
            id: PassId::CallGraph,
            representation: PassRepresentation::Ast,
            dependencies: vec![PassId::SymbolTable],
        };

        scheduler.register_pass(&symbol_table);
        scheduler.register_pass(&type_index);
        scheduler.register_pass(&call_graph);

        let schedule = scheduler.compute_schedule().unwrap();

        assert!(!schedule.needs_ir);
        assert!(!schedule.is_empty());
    }

    #[test]
    fn test_ir_detection() {
        let mut scheduler = PassScheduler::new();

        let ast_pass = MockPass {
            id: PassId::SymbolTable,
            representation: PassRepresentation::Ast,
            dependencies: vec![],
        };
        let ir_pass = MockPass {
            id: PassId::IrCfg,
            representation: PassRepresentation::Ir,
            dependencies: vec![PassId::IrGeneration],
        };

        scheduler.register_pass(&ast_pass);
        assert!(!scheduler.needs_ir());

        scheduler.register_pass(&ir_pass);
        assert!(scheduler.needs_ir());
    }
}
