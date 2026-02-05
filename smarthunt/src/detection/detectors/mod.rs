//! Bug Detectors
//!
//! This module contains all bug detection passes organized by the
//! representation they operate on.
//!
//! # Organization
//!
//! - `ast`: Detectors that operate on the AST representation
//! - `ir`: Detectors that operate on the IR representation (when available)
//! - `hybrid`: Detectors that use both AST and IR
//!
//! # Detector Naming Convention
//!
//! Detectors follow the naming pattern: `<Issue>Detector`
//! Examples: `TxOriginDetector`, `ReentrancyDetector`, `UninitializedStorageDetector`

pub mod ast;
// pub mod ir;     // TODO: Enable when IR is ready
// pub mod hybrid; // TODO: Enable when IR is ready

// Re-export commonly used detectors
pub use ast::*;
