//-------------------------------------------------------------------------
// Parallel Execution Support for SmartHunt
//-------------------------------------------------------------------------

use crate::detectors::Detector;
use crate::engine::context::AnalysisContext;
use bugs::bug::Bug;
use rayon::prelude::*;

/// Configuration for parallel execution.
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Number of worker threads (0 = auto-detect based on CPU cores).
    pub num_threads: usize,

    /// Minimum number of items before parallelization is used.
    /// Below this threshold, sequential execution is used.
    pub min_parallel_size: usize,

    /// Enable work stealing for better load balancing.
    pub work_stealing: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            num_threads: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1),
            min_parallel_size: 4,
            work_stealing: true,
        }
    }
}

impl ParallelConfig {
    /// Create a new parallel config with specified thread count.
    pub fn with_threads(num_threads: usize) -> Self {
        Self {
            num_threads: if num_threads == 0 {
                std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(1)
            } else {
                num_threads
            },
            ..Default::default()
        }
    }

    /// Get the effective number of threads.
    pub fn effective_threads(&self) -> usize {
        if self.num_threads == 0 {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
        } else {
            self.num_threads
        }
    }
}

/// Runner for parallel detector execution.
pub struct DetectorRunner {
    config: ParallelConfig,
}

impl DetectorRunner {
    /// Create a new detector runner with the given configuration.
    pub fn new(config: ParallelConfig) -> Self {
        Self { config }
    }

    /// Create a detector runner with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(ParallelConfig::default())
    }

    /// Run detectors in parallel, returning all detected bugs.
    pub fn run_parallel(
        &self,
        detectors: &[&dyn Detector],
        context: &AnalysisContext,
    ) -> Vec<Bug> {
        if detectors.len() < self.config.min_parallel_size {
            // Not enough work to parallelize
            return self.run_sequential(detectors, context);
        }

        let num_threads = self.config.effective_threads();

        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .expect("Failed to create thread pool");

        pool.install(|| {
            detectors
                .par_iter()
                .flat_map(|detector| {
                    log::debug!("Running detector: {}", detector.id());
                    detector.detect(context)
                })
                .collect()
        })
    }

    /// Run detectors sequentially.
    pub fn run_sequential(
        &self,
        detectors: &[&dyn Detector],
        context: &AnalysisContext,
    ) -> Vec<Bug> {
        detectors
            .iter()
            .flat_map(|detector| {
                log::debug!("Running detector: {}", detector.id());
                detector.detect(context)
            })
            .collect()
    }

    /// Run detectors, choosing parallel or sequential based on configuration.
    pub fn run(
        &self,
        detectors: &[&dyn Detector],
        context: &AnalysisContext,
    ) -> Vec<Bug> {
        if self.config.num_threads > 1 && detectors.len() >= self.config.min_parallel_size {
            self.run_parallel(detectors, context)
        } else {
            self.run_sequential(detectors, context)
        }
    }
}

/// Convenience function to run detectors in parallel.
pub fn run_detectors_parallel(
    detectors: &[&dyn Detector],
    context: &AnalysisContext,
    num_threads: usize,
) -> Vec<Bug> {
    let config = ParallelConfig::with_threads(num_threads);
    let runner = DetectorRunner::new(config);
    runner.run_parallel(detectors, context)
}

/// Convenience function to run detectors with automatic parallelization decision.
pub fn run_detectors(
    detectors: &[&dyn Detector],
    context: &AnalysisContext,
) -> Vec<Bug> {
    let runner = DetectorRunner::with_defaults();
    runner.run(detectors, context)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_config_default() {
        let config = ParallelConfig::default();
        assert!(config.effective_threads() >= 1);
        assert_eq!(config.min_parallel_size, 4);
    }

    #[test]
    fn test_parallel_config_with_threads() {
        let config = ParallelConfig::with_threads(4);
        assert_eq!(config.num_threads, 4);
        assert_eq!(config.effective_threads(), 4);
    }
}
