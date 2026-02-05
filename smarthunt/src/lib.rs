//! SmartHunt - AST-based Smart Contract Bug Detection
//!
//! This crate provides a comprehensive framework for detecting vulnerabilities
//! and code quality issues in Solidity smart contracts using AST analysis.
//!
//! # Architecture
//!
//! SmartHunt uses a pass-based architecture with a built-in analysis framework:
//!
//! - `analysis`: Core analysis framework (migrated from solidity crate)
//!   - `PassManager`: Orchestrates pass registration, scheduling, and execution
//!   - `AnalysisContext`: Central storage for AST, IR, and analysis artifacts
//!   - Pass infrastructure for dependency resolution
//! - `detection`: Pass-based detection framework
//!   - `BugDetectionPass`: Trait for vulnerability detectors
//!   - `DetectionManager`: Orchestrates analysis and detection
//!   - `detectors`: Categorized bug detectors (AST, IR, Hybrid)
//!
//! # Usage
//!
//! ```ignore
//! use smarthunt::detection::{DetectionManager, BugDetectionPass};
//! use smarthunt::AnalysisContext;
//!
//! let mut manager = DetectionManager::new();
//! // Register detectors...
//!
//! let result = manager.run(&mut context);
//! println!("Found {} bugs", result.total_bugs());
//! ```

// Analysis framework (migrated from solidity crate)
pub mod analysis;

// Pass-based detection framework
pub mod detection;

// Output formatting
pub mod output;

// CLI configuration
pub mod config;

// Re-export core analysis types for convenience
pub use analysis::{
    Pass, AnalysisPass, PassId, PassLevel, PassRepresentation,
    AnalysisContext, AnalysisConfig, PassManager, PassManagerConfig,
};

// Re-export from detection framework
pub use detection::{BugDetectionPass, DetectionManager, DetectorRegistry, register_all_detectors};

// Re-export output types
pub use output::{AnalysisReport, OutputFormatter, JsonFormatter, MarkdownFormatter, SarifFormatter};
pub use config::{Config, OutputFormat, SeverityFilter};
