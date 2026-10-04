//! Pipeline Engine
//!
//! The main orchestrator for Verazt Analyzer's two-phase execution:
//!
//! 1. **Analysis Phase**: Run required analysis passes in parallel by
//!    dependency level
//! 2. **Detection Phase**: Run all enabled detectors fully in parallel

use crate::context::AnalysisContext;
use crate::detectors::{BugDetectionPass, DetectorId};
use crate::detectors::base::registry::{DetectorRegistry, register_all_detectors};
use crate::pass_manager::manager::{PassManager, PassManagerConfig};
use crate::pass_manager::PassRegistry;
use crate::passes::register_all_passes;
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
    /// Why the analysis phase failed, if it did. Detectors depending on the
    /// missing analyses then fail too.
    pub analysis_error: Option<String>,
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

    /// One message per failure of the run: the analysis phase error, then
    /// each failed detector. Empty when every phase succeeded, so the
    /// reported bugs are complete.
    pub fn failures(&self) -> Vec<String> {
        let detector_failures = self.detector_stats.iter().filter(|s| !s.success).map(|s| {
            format!(
                "detector '{}' failed: {}",
                s.name,
                s.error.as_deref().unwrap_or("unknown error")
            )
        });
        self.analysis_error.iter().cloned().chain(detector_failures).collect()
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
    /// Analysis passes available to satisfy detector dependencies.
    passes: PassRegistry,
    /// Pipeline configuration.
    config: PipelineConfig,
}

impl PipelineEngine {
    /// Create a new pipeline engine with default detectors registered.
    pub fn new(config: PipelineConfig) -> Self {
        let mut registry = DetectorRegistry::new();
        register_all_detectors(&mut registry);
        Self::with_registry(registry, config)
    }

    /// Create a pipeline engine running the detectors of `registry`, with
    /// all built-in analysis passes available.
    pub fn with_registry(registry: DetectorRegistry, config: PipelineConfig) -> Self {
        let mut passes = PassRegistry::new();
        register_all_passes(&mut passes);
        Self { registry, passes, config }
    }

    /// Get a reference to the detector registry.
    pub fn registry(&self) -> &DetectorRegistry {
        &self.registry
    }

    /// Get a mutable reference to the detector registry.
    pub fn registry_mut(&mut self) -> &mut DetectorRegistry {
        &mut self.registry
    }

    /// Get a mutable reference to the pass registry, to make custom passes
    /// available to custom detectors.
    pub fn passes_mut(&mut self) -> &mut PassRegistry {
        &mut self.passes
    }

    /// Run the full pipeline: analysis phase then detection phase.
    pub fn run(&self, context: &mut AnalysisContext) -> PipelineResult {
        let start = Instant::now();

        // Step 1: Resolve which detectors to run
        let enabled_detectors: Vec<&dyn BugDetectionPass> = self
            .resolve_detectors(context)
            .into_iter()
            .filter(|d| d.is_enabled(context))
            .collect();

        // Step 2: Phase 1 - Analysis passes the detectors depend on
        let analysis_start = Instant::now();
        let analysis_error = self
            .run_analysis_phase(&enabled_detectors, context)
            .map_err(|e| format!("analysis phase failed: {e}"))
            .err();
        if let Some(e) = &analysis_error {
            log::error!("{e}");
        }
        let analysis_duration = analysis_start.elapsed();

        // Step 3: Phase 2 - Detection (parallel)
        let detection_start = Instant::now();
        let (bugs, detector_stats) = self.run_detection_phase(&enabled_detectors, context);
        let detection_duration = detection_start.elapsed();

        // Order bugs deterministically and drop repeated findings
        let bugs = Self::deduplicate_bugs(bugs);

        PipelineResult {
            bugs,
            detector_stats,
            analysis_error,
            analysis_duration,
            detection_duration,
            total_duration: start.elapsed(),
        }
    }

    /// Resolve which detectors should run on `context` based on config. A
    /// detector superseded by another selected one is dropped unless
    /// explicitly enabled.
    ///
    /// Superseding detectors work on BIR, so superseding only applies when
    /// every SIR module of `context` was lowered to BIR: otherwise the
    /// superseded SIR detectors are the only ones covering some modules,
    /// and all are kept.
    fn resolve_detectors(&self, context: &AnalysisContext) -> Vec<&dyn BugDetectionPass> {
        let selected: Vec<&dyn BugDetectionPass> =
            self.registry.all().filter(|d| self.is_detector_enabled(*d)).collect();
        let superseded: HashSet<DetectorId> = if context.bir_covers_sir() {
            selected.iter().flat_map(|d| d.supersedes()).collect()
        } else {
            log::warn!("BIR lowering failed for some modules: keeping superseded SIR detectors");
            HashSet::new()
        };
        selected
            .into_iter()
            .filter(|d| {
                !superseded.contains(&d.meta().id) || is_listed(&self.config.enabled, *d)
            })
            .collect()
    }

    /// Check if a detector is enabled based on config.
    fn is_detector_enabled(&self, detector: &dyn BugDetectionPass) -> bool {
        // Check if explicitly disabled
        if is_listed(&self.config.disabled, detector) {
            return false;
        }

        // If enabled list is non-empty, detector must be in it
        self.config.enabled.is_empty() || is_listed(&self.config.enabled, detector)
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
        let required = self
            .passes
            .instantiate_closure(enabled_detectors.iter().flat_map(|d| d.dependencies()))
            .map_err(|e| e.to_string())?;

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

        for pass in required {
            pass_manager.register_analysis_pass(pass);
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
            Err(e) => Err(e.to_string()),
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

    /// Sort bugs by file, line, column and detector, then drop repeated
    /// reports of one detector at one location (e.g. the same function
    /// reached through several modules).
    ///
    /// Findings of different detectors are all kept, and so are findings
    /// without a known location: they cannot be told apart by location.
    fn deduplicate_bugs(mut bugs: Vec<Bug>) -> Vec<Bug> {
        bugs.sort_by_cached_key(|b| {
            let loc = &b.loc;
            (
                loc.file.clone(),
                loc.start_line,
                loc.start_col,
                loc.end_line,
                loc.end_col,
                b.detector_id.clone(),
            )
        });
        bugs.dedup_by(|a, b| a.loc.is_valid() && a.loc == b.loc && a.detector_id == b.detector_id);
        bugs
    }
}

/// Returns `true` if `list` names the detector by its name or ID.
fn is_listed(list: &[String], detector: &dyn BugDetectionPass) -> bool {
    let meta = detector.meta();
    list.iter().any(|d| d == meta.name || d == meta.id.as_str())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{AnalysisConfig, InputLanguage};
    use crate::passes::bir::FunctionEffectsArtifact;
    use common::loc::Loc;

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
        let detectors = engine.resolve_detectors(&empty_context());
        assert!(!detectors.is_empty());
    }

    #[test]
    fn test_resolve_detectors_filtered() {
        let engine = PipelineEngine::new(PipelineConfig {
            enabled: vec!["tx-origin".to_string()],
            ..PipelineConfig::default()
        });
        let detectors = engine.resolve_detectors(&empty_context());
        assert_eq!(detectors.len(), 1);
    }

    fn empty_context() -> AnalysisContext {
        AnalysisContext::new(vec![], AnalysisConfig::default())
    }

    fn resolved_ids_in(config: PipelineConfig, context: &AnalysisContext) -> Vec<DetectorId> {
        let engine = PipelineEngine::new(config);
        engine.resolve_detectors(context).iter().map(|d| d.meta().id).collect()
    }

    fn resolved_ids(config: PipelineConfig) -> Vec<DetectorId> {
        resolved_ids_in(config, &empty_context())
    }

    #[test]
    fn test_resolve_detectors_keeps_superseded_when_bir_is_partial() {
        let mut context = empty_context();
        // A SIR module whose BIR lowering failed: no BIR unit for it.
        context.sir_units = Some(vec![scirs::sir::Module::new("m", vec![])]);
        let resolved = resolved_ids_in(PipelineConfig::default(), &context);
        assert!(resolved.contains(&DetectorId::ReentrancyFlow));
        assert!(resolved.contains(&DetectorId::Reentrancy));
        assert!(resolved.contains(&DetectorId::CeiViolation));
    }

    fn ids(names: &[&str]) -> Vec<String> {
        names.iter().map(|n| n.to_string()).collect()
    }

    #[test]
    fn test_resolve_detectors_drops_superseded() {
        let resolved = resolved_ids(PipelineConfig::default());
        assert!(resolved.contains(&DetectorId::ReentrancyFlow));
        assert!(!resolved.contains(&DetectorId::Reentrancy));
        assert!(!resolved.contains(&DetectorId::CeiViolation));
    }

    #[test]
    fn test_resolve_detectors_keeps_explicitly_enabled_superseded() {
        let config = PipelineConfig {
            enabled: ids(&["reentrancy", "reentrancy-flow"]),
            ..PipelineConfig::default()
        };
        let resolved = resolved_ids(config);
        assert!(resolved.contains(&DetectorId::Reentrancy));
        assert!(resolved.contains(&DetectorId::ReentrancyFlow));
    }

    #[test]
    fn test_resolve_detectors_restores_superseded_when_superseder_disabled() {
        let config =
            PipelineConfig { disabled: ids(&["reentrancy-flow"]), ..PipelineConfig::default() };
        let resolved = resolved_ids(config);
        assert!(resolved.contains(&DetectorId::Reentrancy));
        assert!(resolved.contains(&DetectorId::CeiViolation));
    }

    #[test]
    fn test_every_detector_dependency_has_a_pass() {
        let engine = PipelineEngine::new(PipelineConfig::default());
        for detector in engine.registry().all() {
            assert!(
                engine.passes.instantiate_closure(detector.dependencies()).is_ok(),
                "'{}' depends on a pass missing from the pass registry",
                detector.name()
            );
        }
    }

    #[test]
    fn test_run_schedules_detector_dependencies() {
        let engine = PipelineEngine::new(PipelineConfig {
            parallel: false,
            enabled: ids(&["reentrancy-flow"]),
            ..PipelineConfig::default()
        });
        let mut context = AnalysisContext::new(vec![], AnalysisConfig::default());
        context.set_bir_units(vec![scirs::bir::Module::new("m".to_string())]);
        engine.run(&mut context);
        assert!(context.has::<FunctionEffectsArtifact>());
    }

    #[test]
    fn test_run_skips_detectors_of_other_platforms() {
        let engine =
            PipelineEngine::new(PipelineConfig { parallel: false, ..PipelineConfig::default() });
        let config =
            AnalysisConfig { input_language: InputLanguage::MoveSui, ..AnalysisConfig::default() };
        let mut context = AnalysisContext::new(vec![], config);
        let result = engine.run(&mut context);
        assert!(result.detector_stats.is_empty());
    }

    fn bug_of(id: &str, loc: Loc) -> Bug {
        let engine = PipelineEngine::new(PipelineConfig::default());
        engine.registry().get(id).expect("built-in detector").meta().bug(None, loc)
    }

    #[test]
    fn test_deduplicate_bugs_keeps_distinct_detectors_at_same_loc() {
        let loc = Loc::new(4, 1, 4, 9);
        let bugs = vec![
            bug_of("tx-origin", loc.clone()),
            bug_of("reentrancy", loc.clone()),
            bug_of("tx-origin", loc),
        ];
        let ids: Vec<_> = PipelineEngine::deduplicate_bugs(bugs)
            .into_iter()
            .map(|b| b.detector_id)
            .collect();
        assert_eq!(ids, ["reentrancy", "tx-origin"]);
    }

    #[test]
    fn test_deduplicate_bugs_keeps_findings_without_loc() {
        let bugs = vec![bug_of("tx-origin", Loc::default()), bug_of("tx-origin", Loc::default())];
        assert_eq!(PipelineEngine::deduplicate_bugs(bugs).len(), 2);
    }

    #[test]
    fn test_deduplicate_bugs_orders_by_line() {
        let bugs = vec![
            bug_of("tx-origin", Loc::new(9, 1, 9, 2)),
            bug_of("tx-origin", Loc::new(2, 1, 2, 2)),
        ];
        let lines: Vec<_> = PipelineEngine::deduplicate_bugs(bugs)
            .into_iter()
            .map(|b| b.loc.start_line)
            .collect();
        assert_eq!(lines, [2, 9]);
    }

    #[test]
    fn test_pipeline_result() {
        let result = PipelineResult::default();
        assert_eq!(result.total_bugs(), 0);
        assert!(!result.has_bugs());
        assert!(result.failures().is_empty());
    }

    #[test]
    fn test_pipeline_result_failures_report_analysis_and_detectors() {
        let ok = DetectorStats { name: "ok".to_string(), success: true, ..Default::default() };
        let failed = DetectorStats {
            name: "broken".to_string(),
            error: Some("Missing required analysis: x".to_string()),
            ..Default::default()
        };
        let result = PipelineResult {
            analysis_error: Some("analysis phase failed: boom".to_string()),
            detector_stats: vec![ok, failed],
            ..Default::default()
        };
        assert_eq!(
            result.failures(),
            [
                "analysis phase failed: boom",
                "detector 'broken' failed: Missing required analysis: x"
            ]
        );
    }
}
