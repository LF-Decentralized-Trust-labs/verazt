//! Pass Executor
//!
//! This module provides the executor that runs passes according to
//! the computed schedule, supporting both sequential and parallel execution.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::{AnalysisPass, PassError, PassExecutionInfo, PassResult};
use crate::analysis::pass_id::PassId;
use crate::analysis::scheduler::{ExecutionLevel, ExecutionSchedule};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

#[cfg(feature = "parallel")]
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

/// Pass executor for running analysis passes.
pub struct PassExecutor {
    /// Configuration.
    config: ExecutorConfig,

    /// Registered passes.
    passes: HashMap<PassId, Arc<dyn AnalysisPass>>,
}

impl Default for PassExecutor {
    fn default() -> Self {
        Self::new(ExecutorConfig::default())
    }
}

impl PassExecutor {
    /// Create a new executor with configuration.
    pub fn new(config: ExecutorConfig) -> Self {
        Self { config, passes: HashMap::new() }
    }

    /// Register a pass.
    pub fn register_pass(&mut self, pass: Arc<dyn AnalysisPass>) {
        self.passes.insert(pass.id(), pass);
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
                            result.pass_id.to_string(),
                            error_msg.clone(),
                        ));
                    }

                    if self.config.fail_fast {
                        return Err(PassError::ExecutionFailed(
                            result.pass_id.to_string(),
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
        level: &ExecutionLevel,
        context: &mut AnalysisContext,
    ) -> PassResult<Vec<PassExecutionInfo>> {
        let mut results = Vec::new();

        // Execute AST passes
        for &pass_id in &level.ast_passes {
            if let Some(result) = self.execute_pass(pass_id, context)? {
                results.push(result);
            }
        }

        // Execute IR passes (only if IR is available)
        if context.has_ir() {
            for &pass_id in &level.ir_passes {
                if let Some(result) = self.execute_pass(pass_id, context)? {
                    results.push(result);
                }
            }
        } else if !level.ir_passes.is_empty() {
            log::warn!("Skipping {} IR passes (IR not available)", level.ir_passes.len());
        }

        // Execute hybrid passes
        for &pass_id in &level.hybrid_passes {
            if let Some(result) = self.execute_pass(pass_id, context)? {
                results.push(result);
            }
        }

        Ok(results)
    }

    /// Execute a single pass.
    fn execute_pass(
        &self,
        pass_id: PassId,
        context: &mut AnalysisContext,
    ) -> PassResult<Option<PassExecutionInfo>> {
        // Skip if already completed
        if context.is_pass_completed(pass_id) {
            context.stats.passes_skipped += 1;
            return Ok(None);
        }

        let pass = self
            .passes
            .get(&pass_id)
            .ok_or_else(|| PassError::PassNotFound(pass_id.to_string()))?;

        let start = Instant::now();
        let name = pass.name().to_string();

        log::info!("Running pass: {} ({})", name, pass_id);

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

    /// Get the number of registered passes.
    pub fn pass_count(&self) -> usize {
        self.passes.len()
    }
}

/// Execute passes in parallel within a level.
#[cfg(feature = "parallel")]
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
