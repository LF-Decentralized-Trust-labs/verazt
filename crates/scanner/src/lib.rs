//! Scanner - Smart Contract Security Vulnerability Scanner
//!
//! This crate provides a comprehensive framework for detecting vulnerabilities
//! and code quality issues in Solidity smart contracts.
//!
//! # Architecture
//!
//! The scanner uses a two-phase pipeline architecture:
//!
//! - `analysis`: Core analysis framework (separate `analysis` crate)
//!   - `PassManager`: Orchestrates pass registration, scheduling, and execution
//!   - `AnalysisContext`: Central storage for AST, IR, and analysis artifacts
//!   - AST passes: SymbolTable, CallGraph, InheritanceGraph, etc.
//! - `pipeline`: Pipeline orchestration
//!   - `PipelineEngine`: Two-phase orchestrator (analysis → detection)
//!   - `BugDetectionPass`: Trait for vulnerability detectors
//!   - `DetectorRegistry`: Manages detector registration and discovery
//! - `engines::pattern`: AST/SIR structural pattern matching engine
//!   - Declarative pattern definitions with captures
//!   - Composable pattern combinators
//!   - Pattern-based bug detectors
//! - `engines::datalog`: (future) Relational reasoning over program facts

// CLI entry module
pub mod cli;

// Detection engines
pub mod engines;

// Bug detectors (grouped by bug category)
pub mod detectors;

// Pipeline orchestration framework
pub mod pipeline;

// Output formatting
pub mod output;

// CLI configuration
pub mod config;

// Re-export core analysis types for convenience
pub use analysis::{
    AnalysisConfig, AnalysisContext, AnalysisPass, Pass, PassId, PassLevel, PassManager,
    PassManagerConfig, PassRepresentation,
};

// Re-export from pipeline framework
pub use pipeline::{
    BugDetectionPass, DetectorRegistry, PipelineConfig, PipelineEngine, PipelineResult,
    register_all_detectors,
};

// Re-export output types
pub use config::{Config, InputLanguage, OutputFormat, SeverityFilter};
pub use output::{
    AnalysisReport, JsonFormatter, MarkdownFormatter, OutputFormatter, SarifFormatter,
};
