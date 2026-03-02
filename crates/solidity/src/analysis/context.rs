//! Analysis Context
//!
//! This module provides the central storage for analysis artifacts,
//! supporting both AST and IR representations.

use crate::analysis::pass_id::PassId;
use crate::ast::SourceUnit;
use crate::ir;
use std::any::Any;
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

    /// Enable IR generation (lazy by default).
    pub enable_ir: bool,

    /// Enable verbose logging.
    pub verbose: bool,

    /// Additional configuration options.
    pub options: HashMap<String, String>,
}

impl AnalysisConfig {
    /// Create a new default configuration.
    pub fn new() -> Self {
        Self {
            enable_parallel: true,
            max_workers: 0, // 0 = auto-detect
            enable_ir: false,
            verbose: false,
            options: HashMap::new(),
        }
    }

    /// Create configuration with parallel execution enabled.
    pub fn parallel() -> Self {
        Self { enable_parallel: true, ..Self::new() }
    }

    /// Create configuration with IR enabled.
    pub fn with_ir() -> Self {
        Self { enable_ir: true, ..Self::new() }
    }
}

/// Statistics about analysis execution.
#[derive(Debug, Clone, Default)]
pub struct AnalysisStats {
    /// Number of AST traversals.
    pub ast_traversals: usize,

    /// Number of IR traversals.
    pub ir_traversals: usize,

    /// Time spent on AST analysis.
    pub ast_analysis_time: Duration,

    /// Time spent on IR analysis.
    pub ir_analysis_time: Duration,

    /// Time spent on IR generation.
    pub ir_generation_time: Duration,

    /// Total passes executed.
    pub passes_executed: usize,

    /// Passes that were skipped (already completed).
    pub passes_skipped: usize,
}

/// The central analysis context holding all artifacts.
///
/// This context stores:
/// - Original AST (always available)
/// - Generated IR (optional, created on demand)
/// - Analysis artifacts from all passes
/// - Execution statistics
#[derive(Debug)]
pub struct AnalysisContext {
    // ========================================
    // Source Representations
    // ========================================
    /// Original AST source units.
    pub source_units: Vec<SourceUnit>,

    /// Generated IR units (optional).
    pub ir_units: Option<Vec<ir::SourceUnit>>,

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
    completed_passes: HashSet<PassId>,

    /// Pass completion order.
    pass_order: Vec<PassId>,

    // ========================================
    // Configuration and Stats
    // ========================================
    /// Analysis configuration.
    pub config: AnalysisConfig,

    /// Execution statistics.
    pub stats: AnalysisStats,
}

impl AnalysisContext {
    /// Create a new analysis context.
    pub fn new(source_units: Vec<SourceUnit>, config: AnalysisConfig) -> Self {
        Self {
            source_units,
            ir_units: None,
            artifacts: HashMap::new(),
            completed_passes: HashSet::new(),
            pass_order: Vec::new(),
            config,
            stats: AnalysisStats::default(),
        }
    }

    /// Create context with IR units.
    pub fn with_ir(mut self, ir_units: Vec<ir::SourceUnit>) -> Self {
        self.ir_units = Some(ir_units);
        self
    }

    // ========================================
    // IR Management
    // ========================================

    /// Check if IR is available.
    pub fn has_ir(&self) -> bool {
        self.ir_units.is_some()
    }

    /// Get IR units (panics if not available).
    pub fn ir_units(&self) -> &Vec<ir::SourceUnit> {
        self.ir_units.as_ref().expect("IR not generated")
    }

    /// Set IR units.
    pub fn set_ir_units(&mut self, ir_units: Vec<ir::SourceUnit>) {
        self.ir_units = Some(ir_units);
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
    pub fn mark_pass_completed(&mut self, pass_id: PassId) {
        if self.completed_passes.insert(pass_id) {
            self.pass_order.push(pass_id);
            self.stats.passes_executed += 1;
        }
    }

    /// Check if a pass has been completed.
    pub fn is_pass_completed(&self, pass_id: PassId) -> bool {
        self.completed_passes.contains(&pass_id)
    }

    /// Get all completed passes in order.
    pub fn completed_passes(&self) -> &[PassId] {
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

    /// Get all contracts from source units.
    pub fn contracts(&self) -> Vec<&crate::ast::ContractDef> {
        self.source_units
            .iter()
            .flat_map(|su| su.elems.iter())
            .filter_map(|elem| {
                if let crate::ast::SourceUnitElem::Contract(c) = elem {
                    Some(c)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all functions from source units.
    pub fn functions(&self) -> Vec<&crate::ast::FuncDef> {
        let mut funcs = Vec::new();

        for su in &self.source_units {
            for elem in &su.elems {
                match elem {
                    crate::ast::SourceUnitElem::Func(f) => funcs.push(f),
                    crate::ast::SourceUnitElem::Contract(c) => {
                        for body_elem in &c.body {
                            if let crate::ast::ContractElem::Func(f) = body_elem {
                                funcs.push(f);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        funcs
    }

    /// Get the number of source units.
    pub fn source_unit_count(&self) -> usize {
        self.source_units.len()
    }

    /// Get execution statistics.
    pub fn stats(&self) -> &AnalysisStats {
        &self.stats
    }

    /// Update AST traversal count.
    pub fn record_ast_traversal(&mut self) {
        self.stats.ast_traversals += 1;
    }

    /// Update IR traversal count.
    pub fn record_ir_traversal(&mut self) {
        self.stats.ir_traversals += 1;
    }
}

impl Clone for AnalysisContext {
    fn clone(&self) -> Self {
        Self {
            source_units: self.source_units.clone(),
            ir_units: self.ir_units.clone(),
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

        assert!(!context.is_pass_completed(PassId::SymbolTable));

        context.mark_pass_completed(PassId::SymbolTable);

        assert!(context.is_pass_completed(PassId::SymbolTable));
        assert_eq!(context.completed_pass_count(), 1);
    }
}
