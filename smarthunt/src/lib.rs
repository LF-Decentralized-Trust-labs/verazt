//! SmartHunt - AST-based Smart Contract Bug Detection
//!
//! This crate provides a comprehensive framework for detecting vulnerabilities
//! and code quality issues in Solidity smart contracts using AST analysis.

pub mod engine;
pub mod tasks;
pub mod passes;
pub mod graph;
pub mod detectors;
pub mod output;

// Re-export main types
pub use engine::context::AnalysisContext;
pub use engine::config::Config;
pub use engine::scheduler::{PassScheduler, create_default_scheduler};
pub use engine::parallel::{ParallelConfig, DetectorRunner, run_detectors, run_detectors_parallel};
pub use detectors::{Detector, DetectorRegistry, create_default_registry};
pub use output::{AnalysisReport, OutputFormatter, JsonFormatter, MarkdownFormatter, SarifFormatter};
pub use passes::{PassId, AnalysisPass, PassError, PassResult};
