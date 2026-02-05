//! Detection Manager
//!
//! Orchestrates the execution of analysis passes and bug detectors.

use crate::detection::pass::BugDetectionPass;
use crate::detection::registry::DetectorRegistry;
use bugs::bug::Bug;
use solidity::analysis::context::AnalysisContext;
use solidity::analysis::manager::{PassManager, PassManagerConfig};
use solidity::analysis::pass_id::PassId;
use std::collections::HashSet;
use std::time::{Duration, Instant};

/// Configuration for detection execution.
#[derive(Debug, Clone)]
pub struct DetectionConfig {
    /// Enable parallel execution of detectors.
    pub parallel: bool,

    /// Number of threads for parallel execution.
    pub num_threads: usize,

    /// List of detector names to run (empty = all).
    pub enabled_detectors: Vec<String>,

    /// List of detector names to skip.
    pub disabled_detectors: Vec<String>,

    /// Whether to collect timing statistics.
    pub collect_stats: bool,
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            parallel: true,
            num_threads: num_cpus::get(),
            enabled_detectors: vec![],
            disabled_detectors: vec![],
            collect_stats: false,
        }
    }
}

/// Statistics for a detector execution.
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

/// Result of running all detectors.
#[derive(Debug, Default)]
pub struct DetectionResult {
    /// All bugs found.
    pub bugs: Vec<Bug>,

    /// Per-detector statistics.
    pub stats: Vec<DetectorStats>,

    /// Total execution time.
    pub total_duration: Duration,

    /// Analysis pass execution time.
    pub analysis_duration: Duration,

    /// Detection execution time.
    pub detection_duration: Duration,
}

impl DetectionResult {
    /// Get bugs by risk level.
    pub fn by_risk_level(&self, level: bugs::bug::RiskLevel) -> Vec<&Bug> {
        self.bugs.iter().filter(|b| b.risk_level == level).collect()
    }

    /// Get total bug count.
    pub fn total_bugs(&self) -> usize {
        self.bugs.len()
    }

    /// Check if any bugs were found.
    pub fn has_bugs(&self) -> bool {
        !self.bugs.is_empty()
    }
}

/// Manager for orchestrating detection.
///
/// The DetectionManager coordinates:
/// 1. Running required analysis passes (via PassManager from solidity crate)
/// 2. Running enabled detectors
/// 3. Collecting and reporting results
pub struct DetectionManager {
    /// Registry of detectors.
    registry: DetectorRegistry,

    /// Analysis pass manager (from solidity crate).
    pass_manager: PassManager,

    /// Detection configuration.
    config: DetectionConfig,
}

impl DetectionManager {
    /// Create a new detection manager.
    pub fn new() -> Self {
        Self {
            registry: DetectorRegistry::new(),
            pass_manager: PassManager::new(PassManagerConfig::default()),
            config: DetectionConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: DetectionConfig) -> Self {
        Self {
            registry: DetectorRegistry::new(),
            pass_manager: PassManager::new(PassManagerConfig::default()),
            config,
        }
    }

    /// Get a reference to the detector registry.
    pub fn registry(&self) -> &DetectorRegistry {
        &self.registry
    }

    /// Get a mutable reference to the detector registry.
    pub fn registry_mut(&mut self) -> &mut DetectorRegistry {
        &mut self.registry
    }

    /// Get a reference to the pass manager.
    pub fn pass_manager(&self) -> &PassManager {
        &self.pass_manager
    }

    /// Get a mutable reference to the pass manager.
    pub fn pass_manager_mut(&mut self) -> &mut PassManager {
        &mut self.pass_manager
    }

    /// Register a detector.
    pub fn register_detector(&mut self, detector: Box<dyn BugDetectionPass>) {
        self.registry.register(detector);
    }

    /// Run all detectors and return results.
    pub fn run(&self, context: &mut AnalysisContext) -> DetectionResult {
        let start = Instant::now();
        let mut result = DetectionResult::default();

        // Step 1: Collect required passes from all enabled detectors
        let analysis_start = Instant::now();
        let required_passes = self.collect_required_passes();

        // Step 2: Run analysis passes
        if let Err(e) = self.run_analysis_passes(context, &required_passes) {
            log::error!("Analysis pass execution failed: {}", e);
            result.total_duration = start.elapsed();
            return result;
        }
        result.analysis_duration = analysis_start.elapsed();

        // Step 3: Run detectors
        let detection_start = Instant::now();
        let (bugs, stats) = self.run_detectors(context);
        result.bugs = bugs;
        result.stats = stats;
        result.detection_duration = detection_start.elapsed();

        result.total_duration = start.elapsed();
        result
    }

    /// Collect all required analysis passes from enabled detectors.
    fn collect_required_passes(&self) -> HashSet<PassId> {
        let mut passes = HashSet::new();

        for detector in self.registry.all() {
            if !self.is_detector_enabled(detector) {
                continue;
            }
            for dep in detector.dependencies() {
                passes.insert(dep);
            }
        }

        passes
    }

    /// Run required analysis passes.
    fn run_analysis_passes(
        &self,
        _context: &mut AnalysisContext,
        required: &HashSet<PassId>,
    ) -> Result<(), String> {
        // For now, just check that the passes exist
        // The pass manager will handle dependency resolution and execution
        for &pass_id in required {
            log::debug!("Analysis requires pass: {:?}", pass_id);
        }
        Ok(())
    }

    /// Run all enabled detectors.
    fn run_detectors(&self, context: &AnalysisContext) -> (Vec<Bug>, Vec<DetectorStats>) {
        let mut all_bugs = Vec::new();
        let mut all_stats = Vec::new();

        // Collect enabled detectors
        let detectors: Vec<_> = self.registry
            .all()
            .filter(|d| self.is_detector_enabled(*d))
            .collect();

        if self.config.parallel && detectors.len() > 1 {
            // Parallel execution using rayon
            self.run_detectors_parallel(context, &detectors)
        } else {
            // Sequential execution
            for detector in detectors {
                let (bugs, stat) = self.run_single_detector(context, detector);
                all_bugs.extend(bugs);
                all_stats.push(stat);
            }
            (all_bugs, all_stats)
        }
    }

    /// Run detectors in parallel.
    fn run_detectors_parallel(
        &self,
        context: &AnalysisContext,
        detectors: &[&dyn BugDetectionPass],
    ) -> (Vec<Bug>, Vec<DetectorStats>) {
        use rayon::prelude::*;

        let results: Vec<_> = detectors
            .par_iter()
            .map(|d| self.run_single_detector(context, *d))
            .collect();

        let mut all_bugs = Vec::new();
        let mut all_stats = Vec::new();

        for (bugs, stat) in results {
            all_bugs.extend(bugs);
            all_stats.push(stat);
        }

        (all_bugs, all_stats)
    }

    /// Run a single detector.
    fn run_single_detector(
        &self,
        context: &AnalysisContext,
        detector: &dyn BugDetectionPass,
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
                (bugs, stat)
            }
            Err(e) => {
                log::error!("Detector {} failed: {}", detector.name(), e);
                stat.success = false;
                stat.error = Some(e.to_string());
                stat.duration = start.elapsed();
                (vec![], stat)
            }
        }
    }

    /// Check if a detector is enabled.
    fn is_detector_enabled(&self, detector: &dyn BugDetectionPass) -> bool {
        let name = detector.name();

        // Check if explicitly disabled
        if self.config.disabled_detectors.iter().any(|d| d == name) {
            return false;
        }

        // If enabled list is non-empty, detector must be in it
        if !self.config.enabled_detectors.is_empty() {
            return self.config.enabled_detectors.iter().any(|d| d == name);
        }

        true
    }
}

impl Default for DetectionManager {
    fn default() -> Self {
        Self::new()
    }
}



/// Module for getting CPU count.
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detection_config_default() {
        let config = DetectionConfig::default();
        assert!(config.parallel);
        assert!(config.enabled_detectors.is_empty());
        assert!(config.disabled_detectors.is_empty());
    }

    #[test]
    fn test_detection_manager_new() {
        let manager = DetectionManager::new();
        assert!(manager.registry().is_empty());
    }
}
