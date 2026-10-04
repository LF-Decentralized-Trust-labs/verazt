//! Pass Scheduler
//!
//! Pure function: takes the registered passes, returns an
//! `ExecutionSchedule`. Must not mutate `AnalysisContext`.

use crate::pass_manager::dependency::DependencyGraph;
use crate::passes::base::{Pass, PassResult};
use std::any::TypeId;

// ========================================================================
// Data Structures
// ========================================================================

/// Schedule of passes to execute.
#[derive(Debug, Clone)]
pub struct ExecutionSchedule {
    /// Levels of passes to execute, in order. The passes of a level depend
    /// only on passes of earlier levels, and are sorted by name.
    pub levels: Vec<Vec<TypeId>>,
}

// ========================================================================
// ExecutionSchedule Implementations
// ========================================================================

impl ExecutionSchedule {
    /// Get total number of passes.
    pub fn total_passes(&self) -> usize {
        self.levels.iter().map(Vec::len).sum()
    }
}

// ========================================================================
// Scheduling
// ========================================================================

/// Compute the execution schedule of `passes`, which must include every
/// pass they depend on.
pub fn compute_schedule<'a, P: Pass + ?Sized + 'a>(
    passes: impl IntoIterator<Item = &'a P>,
) -> PassResult<ExecutionSchedule> {
    let mut graph = DependencyGraph::new();
    for pass in passes {
        graph.add_pass(pass.id(), pass.name(), pass.dependencies());
    }
    Ok(ExecutionSchedule { levels: graph.compute_levels()? })
}

// ========================================================================
// Tests
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::passes::base::meta::{PassLevel, PassRepresentation};

    // Each mock pass is its own type so it gets a unique TypeId.

    struct MockCfgPass;
    impl Pass for MockCfgPass {
        fn name(&self) -> &'static str {
            "MockCfgPass"
        }
        fn description(&self) -> &'static str {
            "Mock CFG pass"
        }
        fn level(&self) -> PassLevel {
            PassLevel::Contract
        }
        fn representation(&self) -> PassRepresentation {
            PassRepresentation::Sir
        }
        fn dependencies(&self) -> Vec<TypeId> {
            vec![]
        }
    }

    struct MockIrCfgPass;
    impl Pass for MockIrCfgPass {
        fn name(&self) -> &'static str {
            "MockIrCfgPass"
        }
        fn description(&self) -> &'static str {
            "Mock IR CFG pass"
        }
        fn level(&self) -> PassLevel {
            PassLevel::Contract
        }
        fn representation(&self) -> PassRepresentation {
            PassRepresentation::Bir
        }
        fn dependencies(&self) -> Vec<TypeId> {
            vec![TypeId::of::<MockCfgPass>()]
        }
    }

    struct MockIrCallGraphPass;
    impl Pass for MockIrCallGraphPass {
        fn name(&self) -> &'static str {
            "MockIrCallGraphPass"
        }
        fn description(&self) -> &'static str {
            "Mock IR call graph pass"
        }
        fn level(&self) -> PassLevel {
            PassLevel::Contract
        }
        fn representation(&self) -> PassRepresentation {
            PassRepresentation::Bir
        }
        fn dependencies(&self) -> Vec<TypeId> {
            vec![TypeId::of::<MockCfgPass>()]
        }
    }

    #[test]
    fn test_schedule_computation() {
        let passes: [&dyn Pass; 3] = [&MockIrCfgPass, &MockIrCallGraphPass, &MockCfgPass];

        let schedule = compute_schedule(passes).unwrap();

        assert_eq!(
            schedule.levels,
            [
                vec![TypeId::of::<MockCfgPass>()],
                vec![TypeId::of::<MockIrCallGraphPass>(), TypeId::of::<MockIrCfgPass>()],
            ]
        );
    }
}
