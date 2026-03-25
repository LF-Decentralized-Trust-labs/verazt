//! Analyzer - Smart Contract Security Vulnerability Analyzer
//!
//! This crate provides a comprehensive framework for detecting vulnerabilities
//! and code quality issues in smart contracts.
//!
//! # Architecture
//!
//! The analyzer uses a two-phase pipeline architecture:
//!
//! - `analysis`: Core analysis framework (merged from the `analysis` crate)
//!   - `PassManager`: Orchestrates pass registration, scheduling, and execution
//!   - `AnalysisContext`: Central storage for AST, IR, and analysis artifacts
//!   - `frameworks/`: Reusable analysis techniques (dfa, cfa, grep, datalog)
//! - `pipeline`: Pipeline orchestration
//!   - `PipelineEngine`: Two-phase orchestrator (analysis → detection)
//! - `detectors`: Bug detection framework and implementations
//!   - `BugDetectionPass`: Trait for vulnerability detectors
//!   - `DetectorRegistry`: Manages detector registration and discovery

// CLI entry module
pub mod cli;

// Typed artifact keys
pub mod artifacts;

// Core analysis framework (merged from crates/analysis)
pub mod analysis;

// Bug detectors (grouped by bug category), with detector identity/traits and
// registry
pub mod detectors;

// Pipeline orchestration framework
pub mod pipeline;

// Output formatting
pub mod output;

// CLI configuration
pub mod config;

// Re-export core analysis types for convenience
pub use crate::analysis::{
    AnalysisConfig, AnalysisContext, AnalysisPass, Pass, PassLevel, PassManager,
    PassManagerConfig, PassRepresentation,
};

// Re-export from detectors framework
pub use detectors::registry::{DetectorRegistry, register_all_detectors};
pub use detectors::{BugDetectionPass, ConfidenceLevel, DetectorId, DetectorResult, create_bug};
pub use pipeline::{PipelineConfig, PipelineEngine, PipelineResult};

// Re-export output types
pub use config::{Config, InputLanguage, OutputFormat, SeverityFilter};
pub use output::{
    AnalysisReport, JsonFormatter, MarkdownFormatter, OutputFormatter, SarifFormatter,
};
