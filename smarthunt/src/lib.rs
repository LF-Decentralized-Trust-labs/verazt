//! SmartHunt - AST-based Smart Contract Bug Detection
//!
//! This crate provides a comprehensive framework for detecting vulnerabilities
//! and code quality issues in Solidity smart contracts using AST analysis.
//!
//! # Architecture
//!
//! SmartHunt uses a pass-based architecture built on top of the analysis
//! framework from the `solidity` crate:
//!
//! - `detection`: New pass-based detection framework
//!   - `BugDetectionPass`: Trait for vulnerability detectors
//!   - `DetectionManager`: Orchestrates analysis and detection
//!   - `detectors`: Categorized bug detectors (AST, IR, Hybrid)
//!
//! - `engine`: Legacy analysis engine (being migrated)
//! - `detectors`: Legacy detector implementations (being migrated)
//!
//! # Usage
//!
//! ```ignore
//! use smarthunt::detection::{DetectionManager, BugDetectionPass};
//! use solidity::analysis::context::AnalysisContext;
//!
//! let mut manager = DetectionManager::new();
//! // Register detectors...
//!
//! let result = manager.run(&mut context);
//! println!("Found {} bugs", result.total_bugs());
//! ```

// New pass-based detection framework
pub mod detection;

// Legacy modules (being migrated to new framework)
pub mod engine;
pub mod tasks;
pub mod passes;
pub mod graph;
pub mod detectors;
pub mod output;

// Re-export from new detection framework
pub use detection::{BugDetectionPass, DetectionManager, DetectorRegistry as NewDetectorRegistry};

// Re-export legacy types (for backward compatibility)
pub use engine::context::AnalysisContext;
pub use engine::config::Config;
pub use engine::scheduler::{PassScheduler, create_default_scheduler};
pub use engine::parallel::{ParallelConfig, DetectorRunner, run_detectors, run_detectors_parallel};
pub use detectors::{Detector, DetectorRegistry, create_default_registry};
pub use output::{AnalysisReport, OutputFormatter, JsonFormatter, MarkdownFormatter, SarifFormatter};
pub use passes::{PassId, AnalysisPass, PassError, PassResult};
