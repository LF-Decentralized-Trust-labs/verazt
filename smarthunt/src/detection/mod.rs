//! Bug Detection Framework
//!
//! This module provides the bug detection framework that extends SmartHunt's
//! analysis framework with vulnerability detection capabilities.
//!
//! # Architecture
//!
//! The detection framework is built on top of SmartHunt's analysis
//! framework, extending it with:
//!
//! - `BugDetectionPass` trait for vulnerability detection passes
//! - `DetectorRegistry` for discovering and managing detectors
//! - `DetectionManager` for orchestrating detection execution
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
//! use smarthunt::detection::{DetectionManager, BugDetectionPass};
//! use smarthunt::AnalysisContext;
//!
//! let mut manager = DetectionManager::new();
//! manager.register_detector(Box::new(TxOriginGrepDetector::new()));
//! manager.register_detector(Box::new(ReentrancyDfaDetector::new()));
//!
//! let bugs = manager.run(&mut context);
//! ```

pub mod manager;
pub mod pass;
pub mod registry;

pub use manager::DetectionManager;
pub use pass::{BugDetectionPass, DetectorResult, create_bug};
pub use registry::{DetectorRegistry, register_all_detectors};
