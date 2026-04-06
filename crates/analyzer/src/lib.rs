//! Analyzer - Smart Contract Security Vulnerability Analyzer
//!
//! This crate provides a comprehensive framework for detecting vulnerabilities
//! and code quality issues in smart contracts.
//!
//! # Architecture
//!
//! The analyzer uses a two-phase pipeline architecture:
//!
//! - `context`: Central storage for AST, IR, and analysis data
//!   (`AnalysisContext`, `ContextKey`)
//! - `pass_manager`: Orchestrates pass registration, scheduling, and execution
//!   (`PassManager`)
//! - `passes`: Concrete analysis passes organized by IR layer (`base/`, `sir/`,
//!   `bir/`, `vir/`)
//! - `frameworks`: Reusable analysis infrastructure (`dfa/`, `cfa/`,
//!   `datalog/`)
//! - `pipeline`: Two-phase orchestrator (analysis → detection;
//!   `PipelineEngine`)
//! - `detectors`: Bug detection framework and implementations
//!   - `base/`: Infrastructure (`BugDetectionPass`, `DetectorId`,
//!     `DetectorRegistry`)
//!   - `scan_adapter`: Wraps `scanner::ScanDetector` → `BugDetectionPass`
//!   - `bir/`: BIR dataflow detectors
//! - `output`: Report formatting (JSON, SARIF, Markdown)

// CLI entry module
pub mod cli;

// Flattened analysis modules (formerly under analysis/)
pub mod context;
pub mod frameworks;
pub mod pass_manager;
pub mod passes;

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
pub use crate::context::{AnalysisConfig, AnalysisContext};
pub use crate::pass_manager::{PassManager, PassManagerConfig};
pub use crate::passes::base::meta::{PassLevel, PassRepresentation};
pub use crate::passes::base::{AnalysisPass, Pass};

// Re-export from detectors framework
pub use detectors::base::registry::{DetectorRegistry, register_all_detectors};
pub use detectors::{BugDetectionPass, ConfidenceLevel, DetectorId, DetectorResult, create_bug};
pub use pipeline::{PipelineConfig, PipelineEngine, PipelineResult};

// Re-export output types
pub use config::{Config, InputLanguage, OutputFormat, SeverityFilter};
pub use output::{
    AnalysisReport, JsonFormatter, MarkdownFormatter, OutputFormatter, SarifFormatter,
};
