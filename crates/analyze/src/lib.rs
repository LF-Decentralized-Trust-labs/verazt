//! SmartHunt - Smart Contract Bug Detection
//!
//! This crate provides a comprehensive framework for detecting vulnerabilities
//! and code quality issues in Solidity smart contracts.
//!
//! # Architecture
//!
//! The analyzer uses a two-phase pipeline architecture:
//!
//! - `analysis`: Core analysis framework
//!   - `PassManager`: Orchestrates pass registration, scheduling, and execution
//!   - `AnalysisContext`: Central storage for AST, IR, and analysis artifacts
//!   - AST passes: SymbolTable, CallGraph, InheritanceGraph, etc.
//! - `pipeline`: Pipeline orchestration
//!   - `PipelineEngine`: Two-phase orchestrator (analysis → detection)
//!   - `BugDetectionPass`: Trait for vulnerability detectors
//!   - `DetectorRegistry`: Manages detector registration and discovery
//! - `absint`: Abstract Interpretation framework
//!   - Generic lattice framework for abstract domains
//!   - Worklist-based fixpoint solver for forward/backward analysis
//!   - Abstract-interpretation-based bug detectors
//! - `grep`: AST Pattern Matching framework
//!   - Declarative pattern definitions with captures
//!   - Composable pattern combinators
//!   - GREP-based bug detectors
//!
//! # Usage
//!
//! ```ignore
//! use analyze::{PipelineEngine, PipelineConfig, AnalysisContext, AnalysisConfig};
//!
//! let engine = PipelineEngine::new(PipelineConfig::default());
//! let mut context = AnalysisContext::new(source_units, AnalysisConfig::default());
//! let result = engine.run(&mut context);
//! println!("Found {} bugs", result.bugs.len());
//! ```

// CLI entry module
pub mod cli;

// Abstract Interpretation framework (standalone)
pub mod absint;

// AST Pattern Matching framework (standalone)
pub mod grep;

// Analysis framework
pub mod analysis;

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
