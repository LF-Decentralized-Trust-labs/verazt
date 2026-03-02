//! Pipeline Engine
//!
//! The main orchestrator for SmartHunt's two-phase execution:
//!
//! 1. **Analysis Phase**: Run required analysis passes in parallel by dependency level
//! 2. **Detection Phase**: Run all enabled detectors fully in parallel

use crate::analysis::context::AnalysisContext;
use crate::analysis::manager::{PassManager, PassManagerConfig};
use crate::analysis::pass::AnalysisPass;
use crate::analysis::pass_id::PassId;
use crate::pipeline::detector::BugDetectionPass;
use crate::pipeline::registry::{DetectorRegistry, register_all_detectors};
use bugs::bug::Bug;
use std::collections::HashSet;
use std::time::{Duration, Instant};

/// Configuration for the pipeline.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Enable parallel execution.
    pub parallel: bool,

    /// Number of worker threads (0 = auto-detect).
    pub num_threads: usize,

    /// List of detector IDs to enable (empty = all).
    pub enabled: Vec<String>,

    /// List of detector IDs to disable.
    pub disabled: Vec<String>,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            parallel: true,
            num_threads: 0,
            enabled: vec![],
            disabled: vec![],
        }
    }
}

/// Statistics for a single detector execution.
#[derive(Debug, Clone, Default)]
pub struct DetectorStats {
    /// Name of the detector.
    pub name: String,
    /// Execution time.
    pub duration: Duration,
    /// Number of bugs found.
    pub bug_count: usize,
    /// Whether execution succeeded.
    pub success: bool,
    /// Error message if failed.
    pub error: Option<String>,
}

/// Result of running the full pipeline.
#[derive(Debug, Default)]
pub struct PipelineResult {
    /// All detected bugs.
    pub bugs: Vec<Bug>,
    /// Per-detector statistics.
    pub detector_stats: Vec<DetectorStats>,
    /// Analysis phase duration.
    pub analysis_duration: Duration,
    /// Detection phase duration.
    pub detection_duration: Duration,
    /// Total pipeline duration.
    pub total_duration: Duration,
}

impl PipelineResult {
    /// Get total bug count.
    pub fn total_bugs(&self) -> usize {
        self.bugs.len()
    }

    /// Check if any bugs were found.
    pub fn has_bugs(&self) -> bool {
        !self.bugs.is_empty()
    }
}

/// The main pipeline engine that orchestrates analysis and detection.
///
/// Execution flow:
///   CLI flags -> resolve detectors -> collect analysis deps
///   -> Phase 1: run analysis passes (parallel by dependency level)
///   -> Phase 2: run detectors (fully parallel)
///   -> collect bugs
pub struct PipelineEngine {
    /// Detector registry.
    registry: DetectorRegistry,
    /// Pipeline configuration.
    config: PipelineConfig,
}

impl PipelineEngine {
    /// Create a new pipeline engine with default detectors registered.
    pub fn new(config: PipelineConfig) -> Self {
        let mut registry = DetectorRegistry::new();
        register_all_detectors(&mut registry);
        Self { registry, config }
    }

    /// Create a pipeline engine with an empty registry (for testing).
    pub fn with_registry(registry: DetectorRegistry, config: PipelineConfig) -> Self {
        Self { registry, config }
    }

    /// Get a reference to the detector registry.
    pub fn registry(&self) -> &DetectorRegistry {
        &self.registry
    }

    /// Get a mutable reference to the detector registry.
    pub fn registry_mut(&mut self) -> &mut DetectorRegistry {
        &mut self.registry
    }

    /// Run the full pipeline: analysis phase then detection phase.
    pub fn run(&self, context: &mut AnalysisContext) -> PipelineResult {
        let start = Instant::now();

        // Step 1: Resolve which detectors to run
        let enabled_detectors = self.resolve_detectors();

        // Step 2: Phase 1 - Analysis
        let analysis_start = Instant::now();
        if let Err(e) = self.run_analysis_phase(&enabled_detectors, context) {
            log::error!("Analysis phase failed: {}", e);
        }
        let analysis_duration = analysis_start.elapsed();

        // Step 3: Phase 2 - Detection (parallel)
        let detection_start = Instant::now();
        let (bugs, detector_stats) = self.run_detection_phase(&enabled_detectors, context);
        let detection_duration = detection_start.elapsed();

        PipelineResult {
            bugs,
            detector_stats,
            analysis_duration,
            detection_duration,
            total_duration: start.elapsed(),
        }
    }

    /// Resolve which detectors should run based on config.
    fn resolve_detectors(&self) -> Vec<&dyn BugDetectionPass> {
        self.registry
            .all()
            .filter(|d| self.is_detector_enabled(*d))
            .collect()
    }

    /// Check if a detector is enabled based on config.
    fn is_detector_enabled(&self, detector: &dyn BugDetectionPass) -> bool {
        let name = detector.name();
        let id = detector.id().as_str();

        // Check if explicitly disabled
        if self.config.disabled.iter().any(|d| d == name || d == id) {
            return false;
        }

        // If enabled list is non-empty, detector must be in it
        if !self.config.enabled.is_empty() {
            return self.config.enabled.iter().any(|d| d == name || d == id);
        }

        true
    }

    // ========================================================================
    // Phase 1: Analysis
    // ========================================================================

    /// Run required analysis passes based on detector dependencies.
    ///
    /// Only passes actually needed by the enabled detectors are scheduled.
    /// Passes are executed in dependency-level order, with passes at the
    /// same level running in parallel.
    fn run_analysis_phase(
        &self,
        enabled_detectors: &[&dyn BugDetectionPass],
        context: &mut AnalysisContext,
    ) -> Result<(), String> {
        // Collect required passes from detector dependencies
        let required: HashSet<PassId> = enabled_detectors
            .iter()
            .flat_map(|d| d.dependencies())
            .collect();

        if required.is_empty() {
            log::debug!("No analysis passes required by enabled detectors");
            return Ok(());
        }

        log::info!("Analysis phase: {} passes required", required.len());

        // Build a PassManager with only the required passes
        let mut pass_manager = PassManager::new(PassManagerConfig {
            enable_parallel: self.config.parallel,
            max_workers: self.config.num_threads,
            fail_fast: true,
            lazy_ir_generation: true,
            verbose: false,
            timing: true,
        });

        // Create and register only the required analysis passes
        // (including transitive dependencies via the pass's own dependencies())
        for &pass_id in &required {
            if let Some(pass) = create_analysis_pass(pass_id) {
                pass_manager.register_analysis_pass(pass);
            }
        }

        // The PassManager handles dependency resolution and parallel execution
        match pass_manager.run(context) {
            Ok(report) => {
                log::info!(
                    "Analysis phase completed: {} passes in {:?}",
                    report.passes_executed,
                    report.total_duration
                );
                Ok(())
            }
            Err(e) => Err(format!("Analysis phase failed: {}", e)),
        }
    }

    // ========================================================================
    // Phase 2: Detection
    // ========================================================================

    /// Run all enabled detectors.
    ///
    /// Detectors read from the immutable AnalysisContext, so they can run
    /// fully in parallel.
    fn run_detection_phase(
        &self,
        enabled_detectors: &[&dyn BugDetectionPass],
        context: &AnalysisContext,
    ) -> (Vec<Bug>, Vec<DetectorStats>) {
        log::info!(
            "Detection phase: {} detectors",
            enabled_detectors.len()
        );

        if self.config.parallel && enabled_detectors.len() > 1 {
            self.run_detectors_parallel(enabled_detectors, context)
        } else {
            self.run_detectors_sequential(enabled_detectors, context)
        }
    }

    /// Run detectors sequentially.
    fn run_detectors_sequential(
        &self,
        detectors: &[&dyn BugDetectionPass],
        context: &AnalysisContext,
    ) -> (Vec<Bug>, Vec<DetectorStats>) {
        let mut all_bugs = Vec::new();
        let mut all_stats = Vec::new();

        for &detector in detectors {
            let (bugs, stat) = run_single_detector(detector, context);
            all_bugs.extend(bugs);
            all_stats.push(stat);
        }

        (all_bugs, all_stats)
    }

    /// Run detectors in parallel using rayon.
    fn run_detectors_parallel(
        &self,
        detectors: &[&dyn BugDetectionPass],
        context: &AnalysisContext,
    ) -> (Vec<Bug>, Vec<DetectorStats>) {
        use rayon::prelude::*;

        let results: Vec<_> = detectors
            .par_iter()
            .map(|&d| run_single_detector(d, context))
            .collect();

        let mut all_bugs = Vec::new();
        let mut all_stats = Vec::new();

        for (bugs, stat) in results {
            all_bugs.extend(bugs);
            all_stats.push(stat);
        }

        (all_bugs, all_stats)
    }
}

/// Run a single detector and collect results.
fn run_single_detector(
    detector: &dyn BugDetectionPass,
    context: &AnalysisContext,
) -> (Vec<Bug>, DetectorStats) {
    let start = Instant::now();
    let mut stat = DetectorStats {
        name: detector.name().to_string(),
        ..Default::default()
    };

    match detector.detect(context) {
        Ok(bugs) => {
            stat.bug_count = bugs.len();
            stat.success = true;
            stat.duration = start.elapsed();
            log::debug!(
                "Detector '{}': {} bugs in {:?}",
                detector.name(),
                bugs.len(),
                stat.duration
            );
            (bugs, stat)
        }
        Err(e) => {
            log::error!("Detector '{}' failed: {}", detector.name(), e);
            stat.success = false;
            stat.error = Some(e.to_string());
            stat.duration = start.elapsed();
            (vec![], stat)
        }
    }
}

/// Create an analysis pass instance from a PassId.
///
/// This factory function maps PassIds to their concrete implementations.
fn create_analysis_pass(pass_id: PassId) -> Option<Box<dyn AnalysisPass>> {
    use crate::analysis::ast::*;
    use crate::analysis::ir::CfgPass;

    match pass_id {
        // AST Foundation Passes
        PassId::SymbolTable => Some(Box::new(SymbolTablePass::new())),
        PassId::TypeIndex => Some(Box::new(TypeIndexPass::new())),

        // AST Analysis Passes
        PassId::InheritanceGraph => Some(Box::new(InheritanceGraphPass::new())),
        PassId::CallGraph => Some(Box::new(CallGraphPass::new())),
        PassId::ModifierAnalysis => Some(Box::new(ModifierAnalysisPass::new())),

        // IR Passes
        PassId::IrCfg => Some(Box::new(CfgPass::new())),

        // Not yet implemented or not an analysis pass
        _ => {
            log::warn!("No analysis pass implementation for {:?}", pass_id);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_config_default() {
        let config = PipelineConfig::default();
        assert!(config.parallel);
        assert!(config.enabled.is_empty());
        assert!(config.disabled.is_empty());
    }

    #[test]
    fn test_pipeline_engine_new() {
        let engine = PipelineEngine::new(PipelineConfig::default());
        assert!(!engine.registry().is_empty());
    }

    #[test]
    fn test_pipeline_engine_with_empty_registry() {
        let engine = PipelineEngine::with_registry(
            DetectorRegistry::new(),
            PipelineConfig::default(),
        );
        assert!(engine.registry().is_empty());
    }

    #[test]
    fn test_resolve_detectors_all() {
        let engine = PipelineEngine::new(PipelineConfig::default());
        let detectors = engine.resolve_detectors();
        assert!(!detectors.is_empty());
    }

    #[test]
    fn test_resolve_detectors_filtered() {
        let engine = PipelineEngine::new(PipelineConfig {
            enabled: vec!["tx-origin".to_string()],
            ..PipelineConfig::default()
        });
        let detectors = engine.resolve_detectors();
        assert_eq!(detectors.len(), 1);
    }

    #[test]
    fn test_create_analysis_pass() {
        assert!(create_analysis_pass(PassId::SymbolTable).is_some());
        assert!(create_analysis_pass(PassId::TypeIndex).is_some());
        assert!(create_analysis_pass(PassId::CallGraph).is_some());
        assert!(create_analysis_pass(PassId::InheritanceGraph).is_some());
        assert!(create_analysis_pass(PassId::ModifierAnalysis).is_some());
    }

    #[test]
    fn test_pipeline_result() {
        let result = PipelineResult::default();
        assert_eq!(result.total_bugs(), 0);
        assert!(!result.has_bugs());
    }
}
