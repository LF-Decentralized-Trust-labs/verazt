//! Pass Executor
//!
//! Borrows the registered passes and runs them on an `AnalysisContext`
//! following an `ExecutionSchedule`, level by level. Passes only read the
//! context and return their artifacts, so the passes of one level run in
//! parallel; their artifacts are stored once the whole level is done.

use crate::context::{AnalysisContext, ErasedArtifact};
use crate::pass_manager::scheduler::ExecutionSchedule;
use crate::passes::base::{ErasedAnalysisPass, PassError, PassExecutionInfo, PassResult};
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use std::any::TypeId;
use std::collections::HashMap;
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

    /// Stop on first error.
    pub fail_fast: bool,

    /// Enable detailed timing.
    pub timing: bool,
}

/// Result of executing a schedule.
#[derive(Debug)]
pub struct ExecutionResult {
    /// Number of passes not run because an earlier run completed them.
    pub already_completed: usize,

    /// Errors encountered.
    pub errors: Vec<PassError>,

    /// Number of failed passes.
    pub failed: usize,

    /// Individual pass execution results.
    pub pass_results: Vec<PassExecutionInfo>,

    /// Number of successful passes.
    pub successful: usize,

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
        Self { parallel: true, max_workers: 0, fail_fast: true, timing: true }
    }
}

// ========================================================================
// ExecutionResult Implementations
// ========================================================================

impl ExecutionResult {
    /// Check if all passes succeeded.
    pub fn is_success(&self) -> bool {
        self.failed == 0
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
    ) -> PassResult<ExecutionResult> {
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
    ) -> PassResult<ExecutionResult> {
        let start = Instant::now();
        let mut result = ExecutionResult {
            already_completed: 0,
            errors: Vec::new(),
            failed: 0,
            pass_results: Vec::new(),
            successful: 0,
            total_duration: Duration::ZERO,
        };

        for (level_idx, level) in schedule.levels.iter().enumerate() {
            log::debug!("Executing level {} ({} passes)", level_idx, level.len());

            let pending: Vec<&dyn ErasedAnalysisPass> = level
                .iter()
                .filter(|&&id| !context.is_pass_completed(id))
                .map(|id| self.passes[id].as_ref())
                .collect();
            result.already_completed += level.len() - pending.len();

            for (info, artifact) in self.run_passes(&pending, context) {
                match artifact {
                    Ok(artifact) => {
                        context.store_erased(artifact);
                        context.mark_pass_completed(info.pass_id);
                        result.successful += 1;
                    }
                    Err(e) => {
                        result.failed += 1;
                        if self.config.fail_fast {
                            return Err(e);
                        }
                        result.errors.push(e);
                    }
                }
                result.pass_results.push(info);
            }
        }

        result.total_duration = start.elapsed();
        Ok(result)
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
