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
//! - **AST Detectors**: Operate on source-level AST
//! - **IR Detectors**: Operate on low-level IR (when available)
//! - **Hybrid Detectors**: Use both AST and IR
//!
//! # Usage
//!
//! ```ignore
//! use smarthunt::detection::{DetectionManager, BugDetectionPass};
//! use smarthunt::AnalysisContext;
//!
//! let mut manager = DetectionManager::new();
//! manager.register_detector(Box::new(TxOriginDetector::new()));
//! manager.register_detector(Box::new(ReentrancyDetector::new()));
//!
//! let bugs = manager.run(&mut context);
//! ```

pub mod pass;
pub mod registry;
pub mod manager;
pub mod detectors;

pub use pass::{BugDetectionPass, DetectorResult, create_bug};
pub use registry::{DetectorRegistry, register_all_detectors};
pub use manager::DetectionManager;
