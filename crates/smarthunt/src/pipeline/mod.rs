//! Pipeline Framework
//!
//! This module provides the pipeline orchestration framework for SmartHunt.
//! It coordinates the two-phase execution:
//!
//! 1. **Analysis Phase**: Run required analysis passes (parallel by dependency level)
//! 2. **Detection Phase**: Run enabled detectors (fully parallel)
//!
//! # Detector Categories
//!
//! Detectors are organized by the representation they operate on:
//!
//! - **DFA Detectors**: Operate on IR using data flow analysis
//! - **GREP Detectors**: Operate on AST using declarative pattern matching
//!
//! # Usage
//!
//! ```ignore
//! use smarthunt::pipeline::{PipelineEngine, PipelineConfig};
//! use smarthunt::AnalysisContext;
//!
//! let engine = PipelineEngine::new(PipelineConfig::default());
//! let result = engine.run(&mut context);
//! println!("Found {} bugs", result.bugs.len());
//! ```

pub mod detector;
pub mod engine;
pub mod registry;

pub use detector::{BugDetectionPass, DetectorResult, create_bug};
pub use engine::{PipelineConfig, PipelineEngine, PipelineResult};
pub use registry::{DetectorRegistry, register_all_detectors};
