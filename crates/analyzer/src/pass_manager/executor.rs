//! Pass Executor
//!
//! Borrows the registered passes and runs them on an `AnalysisContext`
//! following an `ExecutionSchedule`, level by level. Passes only read the
//! context and return their artifacts, so the passes of one level run in
//! parallel; their artifacts are stored once the whole level is done.
//!
//! A failing pass only stops the passes depending on it, directly or
//! transitively: they are skipped, and every other pass still runs.

use crate::context::{AnalysisContext, ErasedArtifact};
use crate::pass_manager::scheduler::ExecutionSchedule;
use crate::passes::base::{ErasedAnalysisPass, PassError, PassExecutionInfo, PassResult};
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

// ========================================================================
// Data Structures
// ========================================================================

/// Configuration for the pass executor.
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Run the passes of a dependency level in parallel.
    pub parallel: bool,

    /// Number of worker threads for parallel execution (0 = one per CPU).
    pub max_workers: usize,

    /// Stop at the first failing pass, instead of skipping only the passes
    /// depending on it.
    pub fail_fast: bool,
}

/// The outcome of running the passes of a schedule, distinct from the bug
/// report `output::AnalysisReport`.
#[derive(Debug, Default)]
pub struct PassRunReport {
    /// Number of passes not run because an earlier run completed them.
    pub already_completed: usize,

    /// Execution information of each pass run, in schedule order.
    pub pass_info: Vec<PassExecutionInfo>,

    /// Names of the passes not run because a pass they depend on,
    /// directly or transitively, failed.
    pub skipped: Vec<String>,

    /// Total execution time.
    pub total_duration: Duration,
}

/// Pass executor for running analysis passes, borrowing the passes of
/// its owner.
pub struct PassExecutor<'a> {
    /// Configuration.
    config: ExecutorConfig,

    /// The passes to run, by pass type.
    passes: &'a HashMap<TypeId, Arc<dyn ErasedAnalysisPass>>,
}

// ========================================================================
// ExecutorConfig Implementations
// ========================================================================

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self { parallel: true, max_workers: 0, fail_fast: false }
    }
}

// ========================================================================
// PassRunReport Implementations
// ========================================================================

impl PassRunReport {
    /// Check if every scheduled pass completed.
    pub fn is_success(&self) -> bool {
        self.skipped.is_empty() && self.pass_info.iter().all(|info| info.success)
    }

    /// The passes that ran and failed.
    pub fn failed(&self) -> impl Iterator<Item = &PassExecutionInfo> {
        self.pass_info.iter().filter(|info| !info.success)
    }

    /// Number of passes that ran and succeeded.
    pub fn passes_executed(&self) -> usize {
        self.pass_info.iter().filter(|info| info.success).count()
    }
}

// ========================================================================
// PassExecutor Implementations
// ========================================================================

impl<'a> PassExecutor<'a> {
    /// Create an executor of `passes` with configuration.
    pub fn new(
        config: ExecutorConfig,
        passes: &'a HashMap<TypeId, Arc<dyn ErasedAnalysisPass>>,
    ) -> Self {
        Self { config, passes }
    }

    /// Execute all passes according to schedule, on `max_workers` threads
    /// when running in parallel.
    pub fn execute(
        &self,
        schedule: &ExecutionSchedule,
        context: &mut AnalysisContext,
    ) -> PassResult<PassRunReport> {
        if self.config.parallel {
            run_on_workers(self.config.max_workers, || self.execute_levels(schedule, context))
        } else {
            self.execute_levels(schedule, context)
        }
    }

    /// Execute the levels of `schedule` in order.
    fn execute_levels(
        &self,
        schedule: &ExecutionSchedule,
        context: &mut AnalysisContext,
    ) -> PassResult<PassRunReport> {
        let start = Instant::now();
        let mut report = PassRunReport::default();
        // Passes that failed or were skipped, whose dependents are skipped.
        let mut incomplete: HashSet<TypeId> = HashSet::new();

        for (level_idx, level) in schedule.levels.iter().enumerate() {
            log::debug!("Executing level {} ({} passes)", level_idx, level.len());

            let mut runnable: Vec<&dyn ErasedAnalysisPass> = Vec::new();
            for &pass_id in level {
                let pass = self.passes[&pass_id].as_ref();
                if context.is_pass_completed(pass_id) {
                    report.already_completed += 1;
                } else if pass.dependencies().iter().any(|dep| incomplete.contains(dep)) {
                    log::warn!("Skipping pass '{}': a pass it depends on failed", pass.name());
                    incomplete.insert(pass_id);
                    report.skipped.push(pass.name().to_string());
                } else {
                    runnable.push(pass);
                }
            }

            for (info, artifact) in self.run_passes(&runnable, context) {
                match artifact {
                    Ok(artifact) => {
                        context.store_erased(artifact);
                        context.mark_pass_completed(info.pass_id);
                    }
                    Err(e) if self.config.fail_fast => {
                        return Err(PassError::ExecutionFailed(info.name, e.to_string()));
                    }
                    Err(e) => {
                        log::error!("Pass '{}' failed: {e}", info.name);
                        incomplete.insert(info.pass_id);
                    }
                }
                report.pass_info.push(info);
            }
        }

        report.total_duration = start.elapsed();
        Ok(report)
    }

    /// Run `passes` on `context`, in parallel if configured, returning
    /// their outcomes in the order of `passes`.
    fn run_passes(
        &self,
        passes: &[&dyn ErasedAnalysisPass],
        context: &AnalysisContext,
    ) -> Vec<(PassExecutionInfo, PassResult<ErasedArtifact>)> {
        if self.config.parallel {
            passes.par_iter().map(|&pass| run_pass(pass, context)).collect()
        } else {
            passes.iter().map(|&pass| run_pass(pass, context)).collect()
        }
    }
}

// ========================================================================
// Helpers
// ========================================================================

/// Run `op` on a thread pool of `workers` threads, or on rayon's global
/// pool (one thread per CPU) when `workers` is 0 or the pool cannot be
/// built.
pub fn run_on_workers<R: Send>(workers: usize, op: impl FnOnce() -> R + Send) -> R {
    if workers == 0 {
        return op();
    }
    match ThreadPoolBuilder::new().num_threads(workers).build() {
        Ok(pool) => pool.install(op),
        Err(e) => {
            log::warn!("Cannot build a pool of {workers} threads, using the global pool: {e}");
            op()
        }
    }
}

/// Run `pass` on `context`, timing it.
fn run_pass(
    pass: &dyn ErasedAnalysisPass,
    context: &AnalysisContext,
) -> (PassExecutionInfo, PassResult<ErasedArtifact>) {
    log::info!("Running pass: {}", pass.name());
    let start = Instant::now();
    let artifact = pass.run_erased(context);
    let info = PassExecutionInfo {
        pass_id: pass.id(),
        name: pass.name().to_string(),
        duration: start.elapsed(),
        success: artifact.is_ok(),
        error: artifact.as_ref().err().map(ToString::to_string),
    };
    (info, artifact)
}
