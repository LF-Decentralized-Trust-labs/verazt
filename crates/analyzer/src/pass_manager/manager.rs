//! Pass Manager
//!
//! Owns the pass registry; entry point for callers; delegates to scheduler
//! then executor; produces `AnalysisReport`. Must not directly touch
//! dependency resolution or execution timing.

use crate::context::AnalysisContext;
use crate::pass_manager::executor::{ExecutorConfig, PassExecutor};
use crate::pass_manager::scheduler::PassScheduler;
use crate::passes::base::{AnalysisPass, PassError, PassExecutionInfo, PassResult};
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// Configuration for the pass manager.
#[derive(Debug, Clone)]
pub struct PassManagerConfig {
    /// Enable parallel execution.
    pub enable_parallel: bool,

    /// Maximum number of worker threads.
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

/// Analysis report containing execution results.
#[derive(Debug)]
pub struct AnalysisReport {
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

impl AnalysisReport {
    /// Create a successful report.
    pub fn success(pass_info: Vec<PassExecutionInfo>, duration: std::time::Duration) -> Self {
        let passes_executed = pass_info.len();
        Self {
            pass_info,
            total_duration: duration,
            passes_executed,
            passes_skipped: 0,
            success: true,
            errors: vec![],
        }
    }

    /// Create a failed report.
    pub fn failure(errors: Vec<String>, duration: std::time::Duration) -> Self {
        Self {
            pass_info: vec![],
            total_duration: duration,
            passes_executed: 0,
            passes_skipped: 0,
            success: false,
            errors,
        }
    }
}

/// The main pass manager.
///
/// The PassManager is responsible for:
/// - Registering analysis passes
/// - Computing execution order based on dependencies
/// - Orchestrating pass execution (sequential or parallel)
/// - Managing the analysis context
pub struct PassManager {
    /// Configuration.
    config: PassManagerConfig,

    /// Registered analysis passes.
    passes: HashMap<TypeId, Arc<dyn AnalysisPass>>,

    /// Pass scheduler.
    scheduler: PassScheduler,

    /// Pass executor.
    executor: PassExecutor,
}

impl Default for PassManager {
    fn default() -> Self {
        Self::new(PassManagerConfig::default())
    }
}

impl PassManager {
    /// Create a new pass manager with configuration.
    pub fn new(config: PassManagerConfig) -> Self {
        let executor_config = ExecutorConfig {
            parallel: config.enable_parallel,
            max_workers: config.max_workers,
            fail_fast: config.fail_fast,
            timing: config.timing,
        };

        Self {
            config,
            passes: HashMap::new(),
            scheduler: PassScheduler::new(),
            executor: PassExecutor::new(executor_config),
        }
    }

    /// Register an analysis pass.
    pub fn register_analysis_pass(&mut self, pass: Box<dyn AnalysisPass>) {
        let pass_arc: Arc<dyn AnalysisPass> = Arc::from(pass);

        // Register with scheduler
        self.scheduler.register_pass(pass_arc.as_ref());

        // Register with executor
        self.executor.register_pass(Arc::clone(&pass_arc));

        // Store in our map
        self.passes.insert(pass_arc.id(), pass_arc);
    }

    /// Register multiple passes.
    pub fn register_passes(&mut self, passes: Vec<Box<dyn AnalysisPass>>) {
        for pass in passes {
            self.register_analysis_pass(pass);
        }
    }

    /// Run all registered passes on the context.
    pub fn run(&mut self, context: &mut AnalysisContext) -> PassResult<AnalysisReport> {
        let start = Instant::now();

        // Compute execution schedule
        let schedule = self.scheduler.compute_schedule()?;

        if self.config.verbose {
            log::info!(
                "Execution schedule: {} levels, {} passes",
                schedule.levels.len(),
                schedule.total_passes(),
            );
        }

        // Execute passes
        let result = self.executor.execute(&schedule, context)?;

        let success = result.is_success();
        let report = AnalysisReport {
            pass_info: result.pass_results,
            total_duration: start.elapsed(),
            passes_executed: result.successful,
            passes_skipped: context.stats.passes_skipped,
            success,
            errors: result.errors.iter().map(|e| e.to_string()).collect(),
        };

        Ok(report)
    }

    /// Run a specific pass and its dependencies.
    pub fn run_pass(&mut self, pass_id: TypeId, context: &mut AnalysisContext) -> PassResult<()> {
        // Get the pass
        let pass = self
            .passes
            .get(&pass_id)
            .ok_or_else(|| PassError::PassNotFound(format!("{:?}", pass_id)))?;

        // Check if already completed
        if context.is_pass_completed(pass_id) {
            return Ok(());
        }

        // Run dependencies first
        for dep in pass.dependencies() {
            if !context.is_pass_completed(dep) {
                self.run_pass(dep, context)?;
            }
        }

        // Run the pass
        let pass = Arc::clone(self.passes.get(&pass_id).unwrap());
        pass.run(context)?;
        context.mark_pass_completed(pass_id);

        Ok(())
    }

    /// Get a registered pass.
    pub fn get_pass(&self, pass_id: TypeId) -> Option<&Arc<dyn AnalysisPass>> {
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
        self.scheduler.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::AnalysisConfig;
    use crate::passes::base::meta::{PassLevel, PassRepresentation};
    use crate::passes::base::traits::Pass;

    // Each mock pass is its own type so it gets a unique TypeId.

    struct MockPassA; // independent pass
    impl Pass for MockPassA {
        fn name(&self) -> &'static str {
            "MockPassA"
        }
        fn description(&self) -> &'static str {
            "A mock pass"
        }
        fn level(&self) -> PassLevel {
            PassLevel::Contract
        }
        fn representation(&self) -> PassRepresentation {
            PassRepresentation::Ir
        }
        fn dependencies(&self) -> Vec<TypeId> {
            vec![]
        }
    }
    impl AnalysisPass for MockPassA {
        fn run(&self, context: &mut AnalysisContext) -> PassResult<()> {
            #[allow(deprecated)]
            context.store_artifact("mock-a", true);
            Ok(())
        }
        fn is_completed(&self, context: &AnalysisContext) -> bool {
            #[allow(deprecated)]
            context.has_artifact("mock-a")
        }
    }

    struct MockPassB; // depends on MockPassA
    impl Pass for MockPassB {
        fn name(&self) -> &'static str {
            "MockPassB"
        }
        fn description(&self) -> &'static str {
            "A mock pass"
        }
        fn level(&self) -> PassLevel {
            PassLevel::Contract
        }
        fn representation(&self) -> PassRepresentation {
            PassRepresentation::Ir
        }
        fn dependencies(&self) -> Vec<TypeId> {
            vec![TypeId::of::<MockPassA>()]
        }
    }
    impl AnalysisPass for MockPassB {
        fn run(&self, context: &mut AnalysisContext) -> PassResult<()> {
            #[allow(deprecated)]
            context.store_artifact("mock-b", true);
            Ok(())
        }
        fn is_completed(&self, context: &AnalysisContext) -> bool {
            #[allow(deprecated)]
            context.has_artifact("mock-b")
        }
    }

    #[test]
    fn test_pass_manager_creation() {
        let manager = PassManager::new(PassManagerConfig::default());
        assert_eq!(manager.pass_count(), 0);
    }

    #[test]
    fn test_pass_registration() {
        let mut manager = PassManager::new(PassManagerConfig::default());

        let pass = MockPassA;

        manager.register_analysis_pass(Box::new(pass));
        assert_eq!(manager.pass_count(), 1);
        assert!(manager.has_pass(TypeId::of::<MockPassA>()));
    }

    #[test]
    fn test_run_passes() {
        let mut manager = PassManager::new(PassManagerConfig::default());

        let pass1 = MockPassA;
        let pass2 = MockPassB;

        manager.register_analysis_pass(Box::new(pass1));
        manager.register_analysis_pass(Box::new(pass2));

        let mut context = AnalysisContext::new(vec![], AnalysisConfig::default());
        let report = manager.run(&mut context).unwrap();

        assert!(report.success);
        assert_eq!(report.passes_executed, 2);
    }
}
