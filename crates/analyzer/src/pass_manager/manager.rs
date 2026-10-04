//! Pass Manager
//!
//! Owns the registered passes; entry point for callers; delegates to
//! scheduler then executor; produces `PassRunReport`. Must not directly
//! touch dependency resolution or execution timing.

use crate::context::AnalysisContext;
use crate::pass_manager::executor::{ExecutorConfig, PassExecutor, PassRunReport};
use crate::pass_manager::scheduler::compute_schedule;
use crate::passes::base::{ErasedAnalysisPass, PassResult};
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

/// Configuration for the pass manager.
#[derive(Debug, Clone)]
pub struct PassManagerConfig {
    /// Run the passes of a dependency level in parallel.
    pub enable_parallel: bool,

    /// Number of worker threads for parallel execution (0 = one per CPU).
    pub max_workers: usize,

    /// Stop at the first failing pass, instead of skipping only the passes
    /// depending on it.
    pub fail_fast: bool,

    /// Enable verbose logging.
    pub verbose: bool,
}

impl Default for PassManagerConfig {
    fn default() -> Self {
        Self {
            enable_parallel: true,
            max_workers: 0, // auto-detect
            fail_fast: false,
            verbose: false,
        }
    }
}

/// The main pass manager.
///
/// The PassManager is responsible for:
/// - Registering analysis passes
/// - Computing execution order based on dependencies
/// - Orchestrating pass execution (sequential or parallel)
pub struct PassManager {
    /// Configuration.
    config: PassManagerConfig,

    /// Registered analysis passes, the only copy: the scheduler and the
    /// executor borrow them for each run.
    passes: HashMap<TypeId, Arc<dyn ErasedAnalysisPass>>,
}

impl Default for PassManager {
    fn default() -> Self {
        Self::new(PassManagerConfig::default())
    }
}

impl PassManager {
    /// Create a new pass manager with configuration.
    pub fn new(config: PassManagerConfig) -> Self {
        Self { config, passes: HashMap::new() }
    }

    /// Register an analysis pass.
    pub fn register_analysis_pass(&mut self, pass: Box<dyn ErasedAnalysisPass>) {
        self.passes.insert(pass.id(), Arc::from(pass));
    }

    /// Register multiple passes.
    pub fn register_passes(&mut self, passes: Vec<Box<dyn ErasedAnalysisPass>>) {
        for pass in passes {
            self.register_analysis_pass(pass);
        }
    }

    /// Run all registered passes on the context.
    ///
    /// A failing pass makes the run skip the passes depending on it, unless
    /// `fail_fast` is set: then the run stops with its error.
    pub fn run(&mut self, context: &mut AnalysisContext) -> PassResult<PassRunReport> {
        // Compute execution schedule
        let schedule = compute_schedule(self.passes.values().map(Arc::as_ref))?;

        if self.config.verbose {
            log::info!(
                "Execution schedule: {} levels, {} passes",
                schedule.levels.len(),
                schedule.total_passes(),
            );
        }

        // Execute passes
        let executor_config = ExecutorConfig {
            parallel: self.config.enable_parallel,
            max_workers: self.config.max_workers,
            fail_fast: self.config.fail_fast,
        };
        PassExecutor::new(executor_config, &self.passes).execute(&schedule, context)
    }

    /// Get a registered pass.
    pub fn get_pass(&self, pass_id: TypeId) -> Option<&Arc<dyn ErasedAnalysisPass>> {
        self.passes.get(&pass_id)
    }

    /// Check if a pass is registered.
    pub fn has_pass(&self, pass_id: TypeId) -> bool {
        self.passes.contains_key(&pass_id)
    }

    /// Get the number of registered passes.
    pub fn pass_count(&self) -> usize {
        self.passes.len()
    }

    /// Get all registered pass IDs.
    pub fn registered_passes(&self) -> Vec<TypeId> {
        self.passes.keys().copied().collect()
    }

    /// Get the configuration.
    pub fn config(&self) -> &PassManagerConfig {
        &self.config
    }

    /// Clear all registered passes.
    pub fn clear(&mut self) {
        self.passes.clear();
    }
}

// ========================================================================
// Tests
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{AnalysisConfig, ContextKey};
    use crate::passes::base::meta::{PassLevel, PassRepresentation};
    use crate::passes::base::traits::{AnalysisPass, Pass, PassError};
    use std::marker::PhantomData;

    /// Artifact of mock pass `P`, recording that it ran.
    struct Ran<P>(PhantomData<P>);

    impl<P: 'static> ContextKey for Ran<P> {
        type Value = ();
        const NAME: &'static str = "ran";
    }

    /// Declare mock pass `$pass`, depending on `$deps` and running `$run`.
    macro_rules! mock_pass {
        ($pass:ident, [$($dep:ty),*], $run:expr) => {
            #[derive(Debug, Default)]
            struct $pass;

            impl Pass for $pass {
                fn name(&self) -> &'static str {
                    stringify!($pass)
                }
                fn description(&self) -> &'static str {
                    "A mock pass"
                }
                fn level(&self) -> PassLevel {
                    PassLevel::Contract
                }
                fn representation(&self) -> PassRepresentation {
                    PassRepresentation::Sir
                }
                fn dependencies(&self) -> Vec<TypeId> {
                    vec![$(TypeId::of::<$dep>()),*]
                }
            }

            impl AnalysisPass for $pass {
                type Artifact = Ran<$pass>;

                fn run(&self, context: &AnalysisContext) -> PassResult<()> {
                    let run: fn(&AnalysisContext) -> PassResult<()> = $run;
                    run(context)
                }
            }
        };
    }

    /// Succeed if pass `P` ran before.
    fn after<P: 'static>(context: &AnalysisContext) -> PassResult<()> {
        if context.has::<Ran<P>>() {
            Ok(())
        } else {
            Err(PassError::ExecutionFailed("mock".to_string(), "dependency".to_string()))
        }
    }

    mock_pass!(MockPassA, [], |_| Ok(()));
    mock_pass!(MockPassB, [MockPassA], after::<MockPassA>);

    fn empty_context() -> AnalysisContext {
        AnalysisContext::new(vec![], AnalysisConfig::default())
    }

    #[test]
    fn test_run_passes_sees_artifacts_of_dependencies() {
        for enable_parallel in [false, true] {
            let config = PassManagerConfig { enable_parallel, max_workers: 2, ..Default::default() };
            let mut manager = PassManager::new(config);
            manager.register_passes(vec![Box::new(MockPassB), Box::new(MockPassA)]);
            let mut context = empty_context();

            let report = manager.run(&mut context).unwrap();

            assert!(report.is_success(), "{report:?}");
            assert_eq!(report.passes_executed(), 2);
            assert!(context.has::<Ran<MockPassB>>());
        }
    }

    mock_pass!(MockFailing, [], |_| Err(PassError::ExecutionFailed(
        "MockFailing".to_string(),
        "boom".to_string()
    )));
    mock_pass!(MockDependent, [MockFailing], |_| Ok(()));
    mock_pass!(MockTransitive, [MockDependent, MockPassA], |_| Ok(()));

    #[test]
    fn test_failing_pass_skips_only_its_dependents() {
        let mut manager = PassManager::default();
        manager.register_passes(vec![
            Box::new(MockFailing),
            Box::new(MockDependent),
            Box::new(MockTransitive),
            Box::new(MockPassA),
            Box::new(MockPassB),
        ]);
        let mut context = empty_context();

        let report = manager.run(&mut context).unwrap();

        let failed: Vec<&str> = report.failed().map(|info| info.name.as_str()).collect();
        assert_eq!(failed, ["MockFailing"]);
        assert_eq!(report.skipped, ["MockDependent", "MockTransitive"]);
        assert!(context.has::<Ran<MockPassA>>());
        assert!(context.has::<Ran<MockPassB>>());
        assert!(!context.has::<Ran<MockDependent>>());
    }

    #[test]
    fn test_fail_fast_stops_with_the_failing_pass() {
        let config = PassManagerConfig { fail_fast: true, ..Default::default() };
        let mut manager = PassManager::new(config);
        manager.register_passes(vec![Box::new(MockFailing), Box::new(MockDependent)]);

        let err = manager.run(&mut empty_context()).unwrap_err();

        assert!(err.to_string().contains("MockFailing"), "{err}");
    }

    #[test]
    fn test_run_does_not_rerun_completed_passes() {
        let mut manager = PassManager::default();
        manager.register_passes(vec![Box::new(MockPassA), Box::new(MockPassB)]);
        let mut context = empty_context();
        manager.run(&mut context).unwrap();

        let report = manager.run(&mut context).unwrap();

        assert!(report.pass_info.is_empty());
        assert_eq!(report.already_completed, 2);
    }
}
