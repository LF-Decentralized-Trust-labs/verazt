//! Minimal stub for old AnalysisContext to support legacy detectors during migration.
//!
//! This module provides a minimal implementation of AnalysisContext that only
//! contains source units, which is sufficient for most legacy detectors.

use solidity::ast::SourceUnit;

/// Minimal analysis context for legacy detector support.
///
/// This is a temporary stub to support old detectors during migration.
/// It only contains source units and no analysis artifacts.
pub struct AnalysisContext {
    /// Source units being analyzed.
    pub source_units: Vec<SourceUnit>,
}

impl AnalysisContext {
    /// Create a new minimal analysis context.
    pub fn new(source_units: Vec<SourceUnit>) -> Self {
        Self { source_units }
    }
}
