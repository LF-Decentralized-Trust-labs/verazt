//! Pass Manager
//!
//! Owns the registered passes; entry point for callers; delegates to
//! scheduler then executor; produces `PassRunReport`. Must not directly
//! touch dependency resolution or execution timing.

use crate::context::AnalysisContext;
use crate::pass_manager::executor::{ExecutorConfig, PassExecutor};
use crate::pass_manager::scheduler::compute_schedule;
use crate::passes::base::{ErasedAnalysisPass, PassExecutionInfo, PassResult};
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// Configuration for the pass manager.
#[derive(Debug, Clone)]
pub struct PassManagerConfig {
    /// Run the passes of a dependency level in parallel.
    pub enable_parallel: bool,

    /// Number of worker threads for parallel execution (0 = one per CPU).
    pub max_workers: usize,

    /// Stop on first error.
    pub fail_fast: bool,

    /// Enable verbose logging.
    pub verbose: bool,

    /// Enable timing information.
    pub timing: bool,
}

impl Default for PassManagerConfig {
    fn default() -> Self {
        Self {
            enable_parallel: true,
            max_workers: 0, // auto-detect
            fail_fast: true,
            verbose: false,
            timing: true,
        }
    }
}

/// The outcome of one `PassManager::run`, distinct from the bug report
/// `output::AnalysisReport`.
#[derive(Debug)]
pub struct PassRunReport {
    /// Pass execution information.
    pub pass_info: Vec<PassExecutionInfo>,

    /// Total analysis duration.
    pub total_duration: std::time::Duration,

    /// Number of passes executed.
    pub passes_executed: usize,

    /// Number of passes skipped (already completed).
    pub passes_skipped: usize,

    /// Whether analysis succeeded.
    pub success: bool,

    /// Error messages.
    pub errors: Vec<String>,
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
    pub fn run(&mut self, context: &mut AnalysisContext) -> PassResult<PassRunReport> {
        let start = Instant::now();

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
            timing: self.config.timing,
        };
        let result = PassExecutor::new(executor_config, &self.passes).execute(&schedule, context)?;

        let success = result.is_success();
        let report = PassRunReport {
            pass_info: result.pass_results,
            total_duration: start.elapsed(),
            passes_executed: result.successful,
            passes_skipped: result.already_completed,
            success,
            errors: result.errors.iter().map(|e| e.to_string()).collect(),
        };

        Ok(report)
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

            assert!(report.success, "{:?}", report.errors);
            assert_eq!(report.passes_executed, 2);
            assert!(context.has::<Ran<MockPassB>>());
        }
    }
}
