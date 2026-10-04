//! Analysis Context
//!
//! This module provides the central storage for analysis artifacts,
//! supporting SIR and BIR representations. AST (frontend) types have
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

// ========================================
// Typed Artifact Key Trait (Step 2.2)
// ========================================

/// Marker trait for type-safe artifact storage and retrieval.
///
/// Each analysis pass that produces an artifact should define a zero-sized
/// marker type implementing this trait. The `Value` associated type pins
/// down the concrete data stored under this key, eliminating stringly-typed
/// lookups and making type mismatches a compile error.
///
/// # Example
///
/// ```ignore
/// pub struct CallGraphArtifact;
/// impl ContextKey for CallGraphArtifact {
///     type Value = CallGraph;
///     const NAME: &'static str = "call_graph"; // debug / serialisation only
/// }
///
/// // Store
/// ctx.store::<CallGraphArtifact>(graph);
///
/// // Retrieve
/// let cg: Option<&CallGraph> = ctx.get::<CallGraphArtifact>();
/// ```
pub trait ContextKey: 'static {
    /// The concrete value type stored under this key.
    type Value: Any + Send + Sync;

    /// A human-readable name, used for logging and serialisation only.
    const NAME: &'static str;
}

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

    /// Time spent on BIR lowering.
    pub air_lowering_time: Duration,

    /// Total passes executed.
    pub passes_executed: usize,

    /// Passes that were skipped (already completed).
    pub passes_skipped: usize,
}

/// The central analysis context holding all data.
///
/// This context stores:
/// - SIR modules (always available when provided)
/// - BIR modules (eagerly lowered from SIR — step 1.8)
/// - Analysis artifacts from all passes
/// - Execution statistics
#[derive(Debug)]
pub struct AnalysisContext {
    // ========================================
    // Source Representations
    // ========================================
    /// SIR modules.
    pub sir_units: Option<Vec<scirs::sir::Module>>,

    /// BIR modules (eagerly lowered from SIR).
    pub bir_units: Option<Vec<scirs::bir::Module>>,

    /// The input source language.
    pub input_language: InputLanguage,

    // ========================================
    // Analysis Artifacts (Dynamic Storage)
    // ========================================
    /// Type-safe artifact storage.
    /// Key: `TypeId` of the `ContextKey` marker, Value: boxed artifact.
    typed_data: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,

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

/// Lower a SIR module through CIR to BIR. A failure is logged and the module
/// is skipped, so BIR detectors silently missing a module stays visible.
fn lower_to_bir(module: &scirs::sir::Module) -> Option<scirs::bir::Module> {
    let cir = scirs::sir::lower::lower_module(module)
        .map_err(|e| log::warn!("SIR → CIR lowering failed for '{}': {e}", module.id))
        .ok()?;
    scirs::cir::lower::lower_module(&cir)
        .map_err(|e| log::warn!("CIR → BIR lowering failed for '{}': {e}", module.id))
        .ok()
}

impl AnalysisContext {
    /// Create a new analysis context from SIR modules.
    ///
    /// BIR modules are **eagerly** lowered from SIR so that all BIR
    /// passes can run without an explicit lowering pass.
    pub fn new(sir_modules: Vec<scirs::sir::Module>, config: AnalysisConfig) -> Self {
        let mut context = Self {
            sir_units: None,
            bir_units: None,
            input_language: config.input_language,
            typed_data: HashMap::new(),
            completed_passes: HashSet::new(),
            pass_order: Vec::new(),
            config,
            stats: AnalysisStats::default(),
        };
        context.set_sir_units(sir_modules);
        context
    }

    // ========================================
    // SIR Management
    // ========================================

    /// Check if SIR is available.
    pub fn has_sir(&self) -> bool {
        self.sir_units.is_some()
    }

    /// Get SIR units (panics if not available).
    pub fn sir_units(&self) -> &Vec<scirs::sir::Module> {
        self.sir_units.as_ref().expect("SIR not available")
    }

    /// Replace the SIR units, eagerly lowering them to BIR. Artifacts and
    /// pass completions computed from the previous units are discarded.
    pub fn set_sir_units(&mut self, sir_units: Vec<scirs::sir::Module>) {
        let bir = sir_units.iter().filter_map(lower_to_bir).collect::<Vec<_>>();
        self.bir_units = (!bir.is_empty()).then_some(bir);
        self.sir_units = (!sir_units.is_empty()).then_some(sir_units);
        self.typed_data.clear();
        self.reset_passes();
    }

    // ========================================
    // BIR Management
    // ========================================

    /// Check if BIR is available.
    pub fn has_bir(&self) -> bool {
        self.bir_units.is_some()
    }

    /// Get BIR units. Returns an empty slice if BIR is not available.
    pub fn bir_units(&self) -> &[scirs::bir::Module] {
        self.bir_units.as_deref().unwrap_or(&[])
    }

    /// Whether every SIR module was lowered to BIR, so that BIR detectors
    /// see all the code SIR detectors see. Lowering maps each SIR module to
    /// at most one BIR module, so equal counts mean none failed.
    pub fn bir_covers_sir(&self) -> bool {
        self.sir_units.as_ref().is_none_or(|sir| sir.len() == self.bir_units().len())
    }

    /// Set BIR units directly (escape hatch).
    pub fn set_bir_units(&mut self, units: Vec<scirs::bir::Module>) {
        self.bir_units = Some(units);
    }

    // ========================================
    // Artifact Storage
    // ========================================

    /// Store a typed artifact using an `ContextKey` marker.
    pub fn store<K: ContextKey>(&mut self, value: K::Value) {
        self.typed_data.insert(TypeId::of::<K>(), Arc::new(value));
    }

    /// Retrieve a typed artifact by key.
    pub fn get<K: ContextKey>(&self) -> Option<&K::Value> {
        self.typed_data
            .get(&TypeId::of::<K>())
            .and_then(|a| a.downcast_ref::<K::Value>())
    }

    /// Retrieve a typed artifact as `Arc` (for sharing across threads).
    pub fn get_arc<K: ContextKey>(&self) -> Option<Arc<K::Value>> {
        self.typed_data
            .get(&TypeId::of::<K>())
            .and_then(|a| Arc::clone(a).downcast::<K::Value>().ok())
    }

    /// Check whether a typed artifact exists.
    pub fn has<K: ContextKey>(&self) -> bool {
        self.typed_data.contains_key(&TypeId::of::<K>())
    }

    /// Remove a typed artifact, returning whether it existed.
    pub fn remove<K: ContextKey>(&mut self) -> bool {
        self.typed_data.remove(&TypeId::of::<K>()).is_some()
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
            sir_units: self.sir_units.clone(),
            bir_units: self.bir_units.clone(),
            input_language: self.input_language,
            typed_data: self.typed_data.clone(),
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
    fn test_typed_artifact_storage() {
        struct TestKey;
        impl ContextKey for TestKey {
            type Value = i32;
            const NAME: &'static str = "test";
        }

        struct OtherKey;
        impl ContextKey for OtherKey {
            type Value = String;
            const NAME: &'static str = "other";
        }

        let mut context = AnalysisContext::new(vec![], AnalysisConfig::default());

        // Store
        context.store::<TestKey>(42);
        assert!(context.has::<TestKey>());
        assert!(!context.has::<OtherKey>());

        // Retrieve
        assert_eq!(context.get::<TestKey>(), Some(&42));
        assert_eq!(context.get::<OtherKey>(), None);

        // Remove
        assert!(context.remove::<TestKey>());
        assert!(!context.has::<TestKey>());
    }

    #[test]
    fn test_set_sir_units_discards_previous_results() {
        struct TestKey;
        impl ContextKey for TestKey {
            type Value = i32;
            const NAME: &'static str = "test";
        }

        let mut context = AnalysisContext::new(vec![], AnalysisConfig::default());
        context.set_bir_units(vec![scirs::bir::Module::new("old".to_string())]);
        context.store::<TestKey>(42);
        context.mark_pass_completed(TypeId::of::<u8>());

        context.set_sir_units(vec![]);

        assert!(!context.has_bir());
        assert!(!context.has::<TestKey>());
        assert!(!context.is_pass_completed(TypeId::of::<u8>()));
        assert!(context.completed_passes().is_empty());
    }

    #[test]
    fn test_pass_completion() {
        let mut context = AnalysisContext::new(vec![], AnalysisConfig::default());

        assert!(!context.is_pass_completed(TypeId::of::<u8>()));

        context.mark_pass_completed(TypeId::of::<u8>());

        assert!(context.is_pass_completed(TypeId::of::<u8>()));
        assert_eq!(context.completed_pass_count(), 1);
    }
}
