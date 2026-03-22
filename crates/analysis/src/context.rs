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
/// impl ArtifactKey for CallGraphArtifact {
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
pub trait ArtifactKey: 'static {
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

/// The central analysis context holding all artifacts.
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
    pub ir_units: Option<Vec<scirs::sir::Module>>,

    /// BIR modules (eagerly lowered from SIR).
    pub air_units: Option<Vec<scirs::bir::Module>>,

    /// The input source language.
    pub input_language: InputLanguage,

    // ========================================
    // Analysis Artifacts (Dynamic Storage)
    // ========================================
    /// Type-erased artifact storage (stringly-typed, deprecated).
    /// Key: artifact name, Value: boxed artifact.
    artifacts: HashMap<String, Arc<dyn Any + Send + Sync>>,

    /// Type-safe artifact storage (step 2.2).
    /// Key: `TypeId` of the `ArtifactKey` marker, Value: boxed artifact.
    typed_artifacts: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,

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
    /// BIR modules are **eagerly** lowered from SIR so that all BIR
    /// passes can run without an explicit `AIRGeneration` dependency.
    pub fn new(sir_modules: Vec<scirs::sir::Module>, config: AnalysisConfig) -> Self {
        let input_language = config.input_language;

        // Eager lowering: SIR → CIR → BIR
        let air_units = if sir_modules.is_empty() {
            None
        } else {
            let start = std::time::Instant::now();
            let bir = sir_modules
                .iter()
                .filter_map(|m| {
                    let cir = scirs::cir::lower::lower_module(m).ok()?;
                    scirs::bir::lower::lower_module(&cir).ok()
                })
                .collect::<Vec<_>>();
            let _elapsed = start.elapsed();
            if bir.is_empty() { None } else { Some(bir) }
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
            typed_artifacts: HashMap::new(),
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

    /// Set IR units and eagerly lower to BIR.
    pub fn set_ir_units(&mut self, ir_units: Vec<scirs::sir::Module>) {
        // Eagerly lower SIR → CIR → BIR
        let bir = ir_units
            .iter()
            .filter_map(|m| {
                let cir = scirs::cir::lower::lower_module(m).ok()?;
                scirs::bir::lower::lower_module(&cir).ok()
            })
            .collect::<Vec<_>>();
        if !bir.is_empty() {
            self.air_units = Some(bir);
        }
        self.ir_units = Some(ir_units);
    }

    // ========================================
    // BIR Management
    // ========================================

    /// Check if BIR is available.
    pub fn has_air(&self) -> bool {
        self.air_units.is_some()
    }

    /// Get BIR units. Returns an empty slice if BIR is not available.
    pub fn air_units(&self) -> &[scirs::bir::Module] {
        self.air_units.as_deref().unwrap_or(&[])
    }

    /// Set BIR units directly (escape hatch).
    pub fn set_air_units(&mut self, units: Vec<scirs::bir::Module>) {
        self.air_units = Some(units);
    }

    // ========================================
    // Artifact Storage (stringly-typed — deprecated)
    // ========================================

    /// Store an artifact in the context (stringly-typed).
    #[deprecated(note = "Use `store::<K>()` with an `ArtifactKey` marker type")]
    pub fn store_artifact<T: Any + Send + Sync>(&mut self, name: &str, artifact: T) {
        self.artifacts.insert(name.to_string(), Arc::new(artifact));
    }

    /// Get an artifact from the context (stringly-typed).
    #[deprecated(note = "Use `get::<K>()` with an `ArtifactKey` marker type")]
    pub fn get_artifact<T: Any + Send + Sync>(&self, name: &str) -> Option<&T> {
        self.artifacts.get(name).and_then(|a| a.downcast_ref::<T>())
    }

    /// Get an artifact as Arc (for sharing) (stringly-typed).
    #[deprecated(note = "Use `get_arc::<K>()` with an `ArtifactKey` marker type")]
    pub fn get_artifact_arc<T: Any + Send + Sync>(&self, name: &str) -> Option<Arc<T>> {
        self.artifacts
            .get(name)
            .and_then(|a| Arc::clone(a).downcast::<T>().ok())
    }

    /// Check if an artifact exists (stringly-typed).
    #[deprecated(note = "Use `has::<K>()` with an `ArtifactKey` marker type")]
    pub fn has_artifact(&self, name: &str) -> bool {
        self.artifacts.contains_key(name)
    }

    /// Remove an artifact (stringly-typed).
    #[deprecated(note = "Use `remove::<K>()` with an `ArtifactKey` marker type")]
    pub fn remove_artifact(&mut self, name: &str) -> bool {
        self.artifacts.remove(name).is_some()
    }

    // ========================================
    // Typed Artifact Storage (Step 2.2)
    // ========================================

    /// Store a typed artifact using an `ArtifactKey` marker.
    pub fn store<K: ArtifactKey>(&mut self, value: K::Value) {
        self.typed_artifacts
            .insert(TypeId::of::<K>(), Arc::new(value));
    }

    /// Retrieve a typed artifact by key.
    pub fn get<K: ArtifactKey>(&self) -> Option<&K::Value> {
        self.typed_artifacts
            .get(&TypeId::of::<K>())
            .and_then(|a| a.downcast_ref::<K::Value>())
    }

    /// Retrieve a typed artifact as `Arc` (for sharing across threads).
    pub fn get_arc<K: ArtifactKey>(&self) -> Option<Arc<K::Value>> {
        self.typed_artifacts
            .get(&TypeId::of::<K>())
            .and_then(|a| Arc::clone(a).downcast::<K::Value>().ok())
    }

    /// Check whether a typed artifact exists.
    pub fn has<K: ArtifactKey>(&self) -> bool {
        self.typed_artifacts.contains_key(&TypeId::of::<K>())
    }

    /// Remove a typed artifact, returning whether it existed.
    pub fn remove<K: ArtifactKey>(&mut self) -> bool {
        self.typed_artifacts.remove(&TypeId::of::<K>()).is_some()
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
            typed_artifacts: self.typed_artifacts.clone(),
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
    #[allow(deprecated)]
    fn test_artifact_storage() {
        let mut context = AnalysisContext::new(vec![], AnalysisConfig::default());

        // Store a simple artifact
        context.store_artifact("test", 42i32);

        assert!(context.has_artifact("test"));
        assert_eq!(context.get_artifact::<i32>("test"), Some(&42));
        assert_eq!(context.get_artifact::<String>("test"), None); // Wrong type
    }

    #[test]
    fn test_typed_artifact_storage() {
        struct TestKey;
        impl ArtifactKey for TestKey {
            type Value = i32;
            const NAME: &'static str = "test";
        }

        struct OtherKey;
        impl ArtifactKey for OtherKey {
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
    fn test_pass_completion() {
        let mut context = AnalysisContext::new(vec![], AnalysisConfig::default());

        assert!(!context.is_pass_completed(TypeId::of::<u8>()));

        context.mark_pass_completed(TypeId::of::<u8>());

        assert!(context.is_pass_completed(TypeId::of::<u8>()));
        assert_eq!(context.completed_pass_count(), 1);
    }
}
