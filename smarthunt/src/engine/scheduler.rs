//-------------------------------------------------------------------------
// Pass Scheduler for SmartHunt
//-------------------------------------------------------------------------

use crate::passes::{
    AnalysisPass, PassId, PassResult, PassError,
    SymbolTablePass, TypeIndexPass, CfgPass, CallGraphPass,
    DataFlowPass, StateMutationPass, AccessControlPass,
};
use crate::engine::context::AnalysisContext;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

/// Result of pass execution with timing information.
#[derive(Debug, Clone)]
pub struct PassExecutionResult {
    pub pass_id: PassId,
    pub pass_name: String,
    pub duration: Duration,
    pub success: bool,
    pub error: Option<String>,
}

/// The pass scheduler coordinates pass execution.
#[derive(Default)]
pub struct PassScheduler {
    /// Registered passes
    passes: HashMap<PassId, Box<dyn AnalysisPass>>,

    /// Execution order (computed from dependencies)
    execution_order: Vec<PassId>,

    /// Execution results
    results: Vec<PassExecutionResult>,
}

impl PassScheduler {
    /// Create a new pass scheduler.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a pass.
    pub fn register(&mut self, pass: Box<dyn AnalysisPass>) {
        let id = pass.id();
        self.passes.insert(id, pass);
    }

    /// Compute execution order based on dependencies.
    pub fn compute_execution_order(&mut self) -> PassResult<()> {
        let mut order = Vec::new();
        let mut visited = HashSet::new();
        let mut in_progress = HashSet::new();

        let pass_ids: Vec<PassId> = self.passes.keys().cloned().collect();

        for pass_id in pass_ids {
            self.topological_sort(pass_id, &mut visited, &mut in_progress, &mut order)?;
        }

        self.execution_order = order;
        Ok(())
    }

    fn topological_sort(
        &self,
        pass_id: PassId,
        visited: &mut HashSet<PassId>,
        in_progress: &mut HashSet<PassId>,
        order: &mut Vec<PassId>,
    ) -> PassResult<()> {
        if visited.contains(&pass_id) {
            return Ok(());
        }

        if in_progress.contains(&pass_id) {
            return Err(PassError::ExecutionFailed(
                pass_id.to_string(),
                "Circular dependency detected".to_string(),
            ));
        }

        in_progress.insert(pass_id);

        if let Some(pass) = self.passes.get(&pass_id) {
            for dep in pass.dependencies() {
                if self.passes.contains_key(&dep) {
                    self.topological_sort(dep, visited, in_progress, order)?;
                }
            }
        }

        in_progress.remove(&pass_id);
        visited.insert(pass_id);
        order.push(pass_id);

        Ok(())
    }

    /// Execute all registered passes in order.
    pub fn execute_all(&mut self, context: &mut AnalysisContext) -> PassResult<()> {
        self.compute_execution_order()?;

        for pass_id in self.execution_order.clone() {
            self.execute_pass(pass_id, context)?;
        }

        Ok(())
    }

    /// Execute a specific pass (and its dependencies).
    pub fn execute_pass(&mut self, pass_id: PassId, context: &mut AnalysisContext) -> PassResult<()> {
        // Check if already completed
        if context.is_pass_completed(&pass_id.to_string()) {
            return Ok(());
        }

        // Execute dependencies first
        if let Some(pass) = self.passes.get(&pass_id) {
            let deps = pass.dependencies();
            for dep in deps {
                if !context.is_pass_completed(&dep.to_string()) {
                    self.execute_pass(dep, context)?;
                }
            }
        }

        // Execute the pass
        let start = Instant::now();

        let result = if let Some(pass) = self.passes.get(&pass_id) {
            let pass_name = pass.name().to_string();
            log::info!("Running pass: {}", pass_name);

            match pass.run(context) {
                Ok(()) => {
                    context.mark_pass_completed(&pass_id.to_string());
                    PassExecutionResult {
                        pass_id,
                        pass_name,
                        duration: start.elapsed(),
                        success: true,
                        error: None,
                    }
                }
                Err(e) => {
                    PassExecutionResult {
                        pass_id,
                        pass_name,
                        duration: start.elapsed(),
                        success: false,
                        error: Some(e.to_string()),
                    }
                }
            }
        } else {
            return Err(PassError::ExecutionFailed(
                pass_id.to_string(),
                "Pass not found".to_string(),
            ));
        };

        let success = result.success;
        let error = result.error.clone();
        self.results.push(result);

        if success {
            Ok(())
        } else {
            Err(PassError::ExecutionFailed(
                pass_id.to_string(),
                error.unwrap_or_else(|| "Unknown error".to_string()),
            ))
        }
    }

    /// Get execution results.
    pub fn get_results(&self) -> &[PassExecutionResult] {
        &self.results
    }

    /// Get total execution time.
    pub fn total_time(&self) -> Duration {
        self.results.iter().map(|r| r.duration).sum()
    }

    /// Print execution summary.
    pub fn print_summary(&self) {
        println!("Pass Execution Summary:");
        println!("========================");

        for result in &self.results {
            let status = if result.success { "✓" } else { "✗" };
            println!(
                "{} {} ({:.2}ms)",
                status,
                result.pass_name,
                result.duration.as_secs_f64() * 1000.0
            );
            if let Some(err) = &result.error {
                println!("  Error: {}", err);
            }
        }

        println!("------------------------");
        println!(
            "Total: {:.2}ms",
            self.total_time().as_secs_f64() * 1000.0
        );
    }
}

/// Create a scheduler with all default passes registered.
pub fn create_default_scheduler() -> PassScheduler {
    let mut scheduler = PassScheduler::new();

    // Phase 1: Context Building (sequential)
    scheduler.register(Box::new(SymbolTablePass::new()));
    scheduler.register(Box::new(TypeIndexPass::new()));
    scheduler.register(Box::new(CfgPass::new()));
    scheduler.register(Box::new(CallGraphPass::new()));

    // Phase 2/3: Inter-procedural Analysis
    scheduler.register(Box::new(DataFlowPass::new()));
    scheduler.register(Box::new(StateMutationPass::new()));
    scheduler.register(Box::new(AccessControlPass::new()));

    scheduler
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_new() {
        let scheduler = PassScheduler::new();
        assert!(scheduler.passes.is_empty());
    }

    #[test]
    fn test_default_scheduler() {
        let scheduler = create_default_scheduler();
        assert!(!scheduler.passes.is_empty());
    }
}

