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

// Legacy modules (kept for backward compatibility during migration)
pub mod detectors;
pub mod output;

// CLI configuration
pub mod config;

// Re-export from new detection framework
pub use detection::{BugDetectionPass, DetectionManager, DetectorRegistry as NewDetectorRegistry};

// Re-export legacy types (for backward compatibility)
pub use detectors::{Detector, DetectorRegistry, create_default_registry, AnalysisContext};
pub use output::{AnalysisReport, OutputFormatter, JsonFormatter, MarkdownFormatter, SarifFormatter};
pub use config::{Config, OutputFormat, SeverityFilter};
