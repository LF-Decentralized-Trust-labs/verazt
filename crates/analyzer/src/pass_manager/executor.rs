//! Pass Executor
//!
//! Takes `ExecutionSchedule` + `AnalysisContext`; drives execution and timing.
//! Must not mutate the pass registry.

use crate::context::AnalysisContext;
use crate::pass_manager::scheduler::ExecutionSchedule;
use crate::passes::base::{AnalysisPass, PassError, PassExecutionInfo, PassResult};
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use rayon::prelude::*;

/// Configuration for the pass executor.
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Enable parallel execution.
    pub parallel: bool,

    /// Maximum number of worker threads.
    pub max_workers: usize,

    /// Stop on first error.
    pub fail_fast: bool,

    /// Enable detailed timing.
    pub timing: bool,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            parallel: true,
            max_workers: 0, // auto-detect
            fail_fast: true,
            timing: true,
        }
    }
}

/// Result of executing a schedule.
#[derive(Debug)]
pub struct ExecutionResult {
    /// Individual pass execution results.
    pub pass_results: Vec<PassExecutionInfo>,

    /// Total execution time.
    pub total_duration: std::time::Duration,

    /// Number of successful passes.
    pub successful: usize,

    /// Number of failed passes.
    pub failed: usize,

    /// Errors encountered.
    pub errors: Vec<PassError>,
}

impl ExecutionResult {
    /// Check if all passes succeeded.
    pub fn is_success(&self) -> bool {
        self.failed == 0
    }
}

/// Pass executor for running analysis passes, borrowing the passes of
/// its owner.
pub struct PassExecutor<'a> {
    /// Configuration.
    config: ExecutorConfig,

    /// The passes to run, by pass type.
    passes: &'a HashMap<TypeId, Arc<dyn AnalysisPass>>,
}

impl<'a> PassExecutor<'a> {
    /// Create an executor of `passes` with configuration.
    pub fn new(config: ExecutorConfig, passes: &'a HashMap<TypeId, Arc<dyn AnalysisPass>>) -> Self {
        Self { config, passes }
    }

    /// Execute all passes according to schedule.
    pub fn execute(
        &self,
        schedule: &ExecutionSchedule,
        context: &mut AnalysisContext,
    ) -> PassResult<ExecutionResult> {
        let start = Instant::now();
        let mut pass_results = Vec::new();
        let mut errors = Vec::new();
        let mut successful = 0;
        let mut failed = 0;

        for (level_idx, level) in schedule.levels.iter().enumerate() {
            log::debug!("Executing level {} ({} passes)", level_idx, level.len());

            let level_results = self.execute_level(level, context)?;

            for result in level_results {
                if result.success {
                    successful += 1;
                } else {
                    failed += 1;
                    if let Some(ref error_msg) = result.error {
                        errors.push(PassError::ExecutionFailed(
                            result.name.clone(),
                            error_msg.clone(),
                        ));
                    }

                    if self.config.fail_fast {
                        return Err(PassError::ExecutionFailed(
                            result.name,
                            result.error.unwrap_or_else(|| "Unknown error".to_string()),
                        ));
                    }
                }
                pass_results.push(result);
            }
        }

        Ok(ExecutionResult {
            pass_results,
            total_duration: start.elapsed(),
            successful,
            failed,
            errors,
        })
    }

    /// Execute a single level of passes.
    fn execute_level(
        &self,
        level: &[TypeId],
        context: &mut AnalysisContext,
    ) -> PassResult<Vec<PassExecutionInfo>> {
        let mut results = Vec::new();
        for &pass_id in level {
            if let Some(result) = self.execute_pass(pass_id, context)? {
                results.push(result);
            }
        }
        Ok(results)
    }

    /// Execute a single pass.
    fn execute_pass(
        &self,
        pass_id: TypeId,
        context: &mut AnalysisContext,
    ) -> PassResult<Option<PassExecutionInfo>> {
        // Skip if already completed
        if context.is_pass_completed(pass_id) {
            context.stats.passes_skipped += 1;
            return Ok(None);
        }

        let pass = &self.passes[&pass_id];
        let start = Instant::now();
        let name = pass.name().to_string();

        log::info!("Running pass: {name}");

        match pass.run(context) {
            Ok(()) => {
                context.mark_pass_completed(pass_id);
                Ok(Some(PassExecutionInfo {
                    pass_id,
                    name,
                    duration: start.elapsed(),
                    success: true,
                    error: None,
                }))
            }
            Err(e) => Ok(Some(PassExecutionInfo {
                pass_id,
                name,
                duration: start.elapsed(),
                success: false,
                error: Some(e.to_string()),
            })),
        }
    }
}

/// Execute passes in parallel within a level.
pub fn execute_level_parallel(
    passes: &[Arc<dyn AnalysisPass>],
    context: &AnalysisContext,
) -> Vec<PassResult<PassExecutionInfo>> {
    passes
        .par_iter()
        .map(|pass| {
            let start = Instant::now();
            let mut local_context = context.clone();

            match pass.run(&mut local_context) {
                Ok(()) => Ok(PassExecutionInfo {
                    pass_id: pass.id(),
                    name: pass.name().to_string(),
                    duration: start.elapsed(),
                    success: true,
                    error: None,
                }),
                Err(e) => Ok(PassExecutionInfo {
                    pass_id: pass.id(),
                    name: pass.name().to_string(),
                    duration: start.elapsed(),
                    success: false,
                    error: Some(e.to_string()),
                }),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_config_default() {
        let config = ExecutorConfig::default();
        assert!(config.parallel);
        assert!(config.fail_fast);
        assert!(config.timing);
    }

    #[test]
    fn test_execution_result() {
        let result = ExecutionResult {
            pass_results: vec![],
            total_duration: std::time::Duration::from_millis(100),
            successful: 5,
            failed: 0,
            errors: vec![],
        };
        assert!(result.is_success());
    }
}
