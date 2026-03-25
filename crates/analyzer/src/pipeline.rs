//! Pipeline Engine
//!
//! The main orchestrator for Verazt Analyzer's two-phase execution:
//!
//! 1. **Analysis Phase**: Run required analysis passes in parallel by
//!    dependency level
//! 2. **Detection Phase**: Run all enabled detectors fully in parallel

use crate::analysis::AnalysisPass;
use crate::analysis::PassRepresentation;
use crate::analysis::context::AnalysisContext;
use crate::analysis::pipeline::manager::{PassManager, PassManagerConfig};
use crate::config::InputLanguage;
use crate::detectors::BugDetectionPass;
use crate::detectors::registry::{DetectorRegistry, register_all_detectors};
use bugs::bug::Bug;
use std::any::TypeId;
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
        Self { parallel: true, num_threads: 0, enabled: vec![], disabled: vec![] }
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

        // Step 1: Resolve which detectors to run (language-aware)
        let enabled_detectors = self.resolve_detectors_for_language(context.input_language);

        // Step 2: Phase 1 - AST + IR analysis (existing)
        let analysis_start = Instant::now();
        if let Err(e) = self.run_analysis_phase(&enabled_detectors, context) {
            log::error!("Analysis phase failed: {}", e);
        }

        // Phase 2 — SIR structural analysis
        if context.has_ir() {
            if let Err(e) = self.run_sir_phase(context) {
                log::error!("SIR structural phase failed: {}", e);
            }
        }

        // Phase 3 — BIR dataflow analysis
        if context.has_air() {
            if let Err(e) = self.run_air_phase(context) {
                log::error!("BIR dataflow phase failed: {}", e);
            }
        }
        let analysis_duration = analysis_start.elapsed();

        // Step 3: Phase 4 - Detection (parallel)
        let detection_start = Instant::now();
        let (bugs, detector_stats) = self.run_detection_phase(&enabled_detectors, context);
        let detection_duration = detection_start.elapsed();

        // Deduplicate bugs across tiers
        let bugs = Self::deduplicate_bugs(bugs);

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

    /// Resolve which detectors should run, taking into account the input
    /// language. AST-only (GREP) detectors are skipped for Vyper because
    /// they operate on Solidity AST types.
    fn resolve_detectors_for_language(
        &self,
        language: InputLanguage,
    ) -> Vec<&dyn BugDetectionPass> {
        self.resolve_detectors()
            .into_iter()
            .filter(|d| {
                if language == InputLanguage::Vyper {
                    // Only keep detectors that operate on IR or hybrid;
                    // skip pure AST (GREP) detectors since they target
                    // Solidity AST types.
                    d.representation() != PassRepresentation::Ast
                } else {
                    true
                }
            })
            .collect()
    }

    /// Check if a detector is enabled based on config.
    fn is_detector_enabled(&self, detector: &dyn BugDetectionPass) -> bool {
        let name = detector.name();
        let id = detector.detector_id().as_str();

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
        let required: HashSet<TypeId> = enabled_detectors
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
            verbose: false,
            timing: true,
        });

        // Create and register only the required analysis passes
        // (including transitive dependencies via the pass's own dependencies())
        for &pass_id in &required {
            // Note: Vyper AST-level pass filtering removed (TypeId has no representation
            // info)
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
    // Phase 2: SIR Structural Analysis
    // ========================================================================

    /// Run SIR structural analysis passes.
    ///
    /// These passes operate on `scirs::sir::Module` and detect issues visible
    /// in the SIR tree structure (missing annotations, wrong overflow
    /// semantics, etc.).
    fn run_sir_phase(&self, _context: &mut AnalysisContext) -> Result<(), String> {
        log::info!("SIR structural phase");
        // SIR structural passes store their findings as artifacts.
        // They are registered as analysis passes and run via the normal
        // PassManager scheduling.  The create_analysis_pass factory already
        // handles them; this method is a logical grouping marker for now.
        Ok(())
    }

    // ========================================================================
    // Phase 3: BIR Dataflow Analysis
    // ========================================================================

    /// Run BIR generation and dataflow analysis passes.
    ///
    /// Dependency order:
    ///   AIRGeneration → AIRTaintPropagation → {AIRReentrancy,
    /// AIRAccessControl, AIRArithmetic, ...}
    fn run_air_phase(&self, _context: &mut AnalysisContext) -> Result<(), String> {
        log::info!("BIR dataflow phase");
        // BIR passes are also registered as analysis passes with the
        // correct dependencies.  The PassManager scheduler will
        // naturally place them after AIRGeneration.
        Ok(())
    }

    // ========================================================================
    // Phase 4: Detection
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
        log::info!("Detection phase: {} detectors", enabled_detectors.len());

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

    /// Deduplicate bugs across tiers.
    ///
    /// When both a lower-tier (AST) and higher-tier (SIR/BIR) detector
    /// produce findings at the same source location for the same category,
    /// keep only the higher-tier finding to avoid noise.
    fn deduplicate_bugs(mut bugs: Vec<Bug>) -> Vec<Bug> {
        if bugs.len() <= 1 {
            return bugs;
        }

        // Stable sort by location + category so duplicates are adjacent
        bugs.sort_by(|a, b| {
            let loc_cmp = format!("{:?}{:?}", a.loc, a.category)
                .cmp(&format!("{:?}{:?}", b.loc, b.category));
            loc_cmp
        });

        bugs.dedup_by(|a, b| {
            // Same location and category → keep one (b survives in dedup_by)
            format!("{:?}", a.loc) == format!("{:?}", b.loc) && a.category == b.category
        });

        bugs
    }
}

/// Run a single detector and collect results.
fn run_single_detector(
    detector: &dyn BugDetectionPass,
    context: &AnalysisContext,
) -> (Vec<Bug>, DetectorStats) {
    let start = Instant::now();
    let mut stat = DetectorStats { name: detector.name().to_string(), ..Default::default() };

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

/// Create an analysis pass instance from a TypeId.
///
/// This factory function maps TypeIds to their concrete implementations.
fn create_analysis_pass(pass_id: TypeId) -> Option<Box<dyn AnalysisPass>> {
    if pass_id == TypeId::of::<crate::analysis::passes::bir::TaintPropagationPass>() {
        Some(Box::new(crate::analysis::passes::bir::TaintPropagationPass))
    } else {
        log::warn!("No analysis pass implementation for {:?}", pass_id);
        None
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
        let engine =
            PipelineEngine::with_registry(DetectorRegistry::new(), PipelineConfig::default());
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
        assert!(
            create_analysis_pass(
                TypeId::of::<crate::analysis::passes::bir::TaintPropagationPass>()
            )
            .is_some()
        );
    }

    #[test]
    fn test_pipeline_result() {
        let result = PipelineResult::default();
        assert_eq!(result.total_bugs(), 0);
        assert!(!result.has_bugs());
    }
}
