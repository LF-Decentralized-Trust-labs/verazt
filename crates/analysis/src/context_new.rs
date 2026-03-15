//! Analysis Context
//!
//! This module provides the central storage for analysis artifacts,
//! supporting SIR and AIR representations. AST (frontend) types have
//! been removed — all input is via SIR `Module`.

/// The input source language.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum InputLanguage {
    #[default]
    Solidity,
    Vyper,
    MoveSui,
    MoveAptos,
    Solana,
}

use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

/// Configuration for analysis.
#[derive(Debug, Clone, Default)]
pub struct AnalysisConfig {
    /// Enable parallel execution.
    pub enable_parallel: bool,

    /// Maximum number of worker threads.
    pub max_workers: usize,

    /// Enable verbose logging.
    pub verbose: bool,

    /// The input source language.
    pub input_language: InputLanguage,

    /// Additional configuration options.
    pub options: HashMap<String, String>,
}

impl AnalysisConfig {
    /// Create a new default configuration.
    pub fn new() -> Self {
        Self {
            enable_parallel: true,
            max_workers: 0, // 0 = auto-detect
            verbose: false,
            input_language: InputLanguage::default(),
            options: HashMap::new(),
        }
    }

    /// Create configuration with parallel execution enabled.
    pub fn parallel() -> Self {
        Self { enable_parallel: true, ..Self::new() }
    }
}

/// Statistics about analysis execution.
#[derive(Debug, Clone, Default)]
pub struct AnalysisStats {
    /// Number of IR traversals.
    pub ir_traversals: usize,

    /// Time spent on IR analysis.
    pub ir_analysis_time: Duration,

    /// Time spent on AIR lowering.
    pub air_lowering_time: Duration,

    /// Total passes executed.
    pub passes_executed: usize,

    /// Passes that were skipped (already completed).
    pub passes_skipped: usize,
}

/// The central analysis context holding all artifacts.
///
/// This context stores:
/// - SIR modules (always available when provided)
/// - AIR modules (eagerly lowered from SIR — step 1.8)
/// - Analysis artifacts from all passes
/// - Execution statistics
#[derive(Debug)]
pub struct AnalysisContext {
    // ========================================
    // Source Representations
    // ========================================
    /// SIR modules.
    pub ir_units: Option<Vec<scirs::sir::Module>>,

    /// AIR modules (eagerly lowered from SIR).
    pub air_units: Option<Vec<scirs::air::Module>>,

    /// The input source language.
    pub input_language: InputLanguage,

    // ========================================
    // Analysis Artifacts (Dynamic Storage)
    // ========================================
    /// Type-erased artifact storage.
    /// Key: artifact name, Value: boxed artifact.
    artifacts: HashMap<String, Arc<dyn Any + Send + Sync>>,

    // ========================================
    // Pass Management
    // ========================================
    /// Set of completed pass IDs.
    completed_passes: HashSet<TypeId>,

    /// Pass completion order.
    pass_order: Vec<TypeId>,

    // ========================================
    // Configuration and Stats
    // ========================================
    /// Analysis configuration.
    pub config: AnalysisConfig,

    /// Execution statistics.
    pub stats: AnalysisStats,
}

impl AnalysisContext {
    /// Create a new analysis context from SIR modules.
    ///
    /// AIR modules are **eagerly** lowered from SIR so that all AIR
    /// passes can run without an explicit `AIRGeneration` dependency.
    pub fn new(sir_modules: Vec<scirs::sir::Module>, config: AnalysisConfig) -> Self {
        let input_language = config.input_language;

        // Eager lowering: SIR → CIR → AIR
        let air_units = if sir_modules.is_empty() {
            None
        } else {
            let start = std::time::Instant::now();
            let air = sir_modules
                .iter()
                .filter_map(|m| {
                    let cir = scirs::cir::lower::lower_module(m).ok()?;
                    scirs::air::lower::lower_module(&cir).ok()
                })
                .collect::<Vec<_>>();
            let _elapsed = start.elapsed();
            if air.is_empty() { None } else { Some(air) }
        };

        let ir_units = if sir_modules.is_empty() {
            None
        } else {
            Some(sir_modules)
        };

        Self {
            ir_units,
            air_units,
            input_language,
            artifacts: HashMap::new(),
            completed_passes: HashSet::new(),
            pass_order: Vec::new(),
            config,
            stats: AnalysisStats::default(),
        }
    }

    // ========================================
    // IR Management
    // ========================================

    /// Check if IR is available.
    pub fn has_ir(&self) -> bool {
        self.ir_units.is_some()
    }

    /// Get IR units (panics if not available).
    pub fn ir_units(&self) -> &Vec<scirs::sir::Module> {
        self.ir_units.as_ref().expect("IR not generated")
    }

    /// Set IR units and eagerly lower to AIR.
    pub fn set_ir_units(&mut self, ir_units: Vec<scirs::sir::Module>) {
        // Eagerly lower SIR → CIR → AIR
        let air = ir_units
            .iter()
            .filter_map(|m| {
                let cir = scirs::cir::lower::lower_module(m).ok()?;
                scirs::air::lower::lower_module(&cir).ok()
            })
            .collect::<Vec<_>>();
        if !air.is_empty() {
            self.air_units = Some(air);
        }
        self.ir_units = Some(ir_units);
    }

    // ========================================
    // AIR Management
    // ========================================

    /// Check if AIR is available.
    pub fn has_air(&self) -> bool {
        self.air_units.is_some()
    }

    /// Get AIR units (panics if not available).
    pub fn air_units(&self) -> &Vec<scirs::air::Module> {
        self.air_units.as_ref().expect("AIR not generated")
    }

    /// Set AIR units directly (escape hatch).
    pub fn set_air_units(&mut self, units: Vec<scirs::air::Module>) {
        self.air_units = Some(units);
    }

    // ========================================
    // Artifact Storage
    // ========================================

    /// Store an artifact in the context.
    pub fn store_artifact<T: Any + Send + Sync>(&mut self, name: &str, artifact: T) {
        self.artifacts.insert(name.to_string(), Arc::new(artifact));
    }

    /// Get an artifact from the context.
    pub fn get_artifact<T: Any + Send + Sync>(&self, name: &str) -> Option<&T> {
        self.artifacts.get(name).and_then(|a| a.downcast_ref::<T>())
    }

    /// Get an artifact as Arc (for sharing).
    pub fn get_artifact_arc<T: Any + Send + Sync>(&self, name: &str) -> Option<Arc<T>> {
        self.artifacts
            .get(name)
            .and_then(|a| Arc::clone(a).downcast::<T>().ok())
    }

    /// Check if an artifact exists.
    pub fn has_artifact(&self, name: &str) -> bool {
        self.artifacts.contains_key(name)
    }

    /// Remove an artifact.
    pub fn remove_artifact(&mut self, name: &str) -> bool {
        self.artifacts.remove(name).is_some()
    }

    // ========================================
    // Pass Management
    // ========================================

    /// Mark a pass as completed.
    pub fn mark_pass_completed(&mut self, pass_id: TypeId) {
        if self.completed_passes.insert(pass_id) {
            self.pass_order.push(pass_id);
            self.stats.passes_executed += 1;
        }
    }

    /// Check if a pass has been completed.
    pub fn is_pass_completed(&self, pass_id: TypeId) -> bool {
        self.completed_passes.contains(&pass_id)
    }

    /// Get all completed passes in order.
    pub fn completed_passes(&self) -> &[TypeId] {
        &self.pass_order
    }

    /// Get the number of completed passes.
    pub fn completed_pass_count(&self) -> usize {
        self.completed_passes.len()
    }

    /// Reset pass completion status.
    pub fn reset_passes(&mut self) {
        self.completed_passes.clear();
        self.pass_order.clear();
    }

    // ========================================
    // Convenience Methods
    // ========================================

    /// Get execution statistics.
    pub fn stats(&self) -> &AnalysisStats {
        &self.stats
    }

    /// Update IR traversal count.
    pub fn record_ir_traversal(&mut self) {
        self.stats.ir_traversals += 1;
    }
}

impl Clone for AnalysisContext {
    fn clone(&self) -> Self {
        Self {
            ir_units: self.ir_units.clone(),
            air_units: self.air_units.clone(),
            input_language: self.input_language,
            artifacts: self.artifacts.clone(),
            completed_passes: self.completed_passes.clone(),
            pass_order: self.pass_order.clone(),
            config: self.config.clone(),
            stats: self.stats.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_storage() {
        let mut context = AnalysisContext::new(vec![], AnalysisConfig::default());

        // Store a simple artifact
        context.store_artifact("test", 42i32);

        assert!(context.has_artifact("test"));
        assert_eq!(context.get_artifact::<i32>("test"), Some(&42));
        assert_eq!(context.get_artifact::<String>("test"), None); // Wrong type
    }

    #[test]
    fn test_pass_completion() {
        let mut context = AnalysisContext::new(vec![], AnalysisConfig::default());

        // Use an arbitrary type as a stand-in pass ID
        let pass_id = TypeId::of::<u8>();

        assert!(!context.is_pass_completed(pass_id));

        context.mark_pass_completed(pass_id);

        assert!(context.is_pass_completed(pass_id));
        assert_eq!(context.completed_pass_count(), 1);
    }
}
