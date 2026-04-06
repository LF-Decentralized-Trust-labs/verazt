//! Scanner — Syntactic Pattern-Matching Security Detectors
//!
//! Fast, lightweight security checks that operate on the SIR
//! representation via a single-pass tree walk. No dataflow,
//! control-flow, or Datalog frameworks are used.
//!
//! ## Architecture
//!
//! The `ScanEngine` walks the SIR hierarchy (Module → Contract →
//! Function) exactly **once**, dispatching to detectors at each level.
//! This eliminates the 21× redundant traversals of the previous design.
//!
//! ## Organization
//!
//! Detectors are grouped by dialect and detection level:
//!
//! - `detectors/evm/module/`    — Module-level detectors
//! - `detectors/evm/contract/`  — Contract-level detectors
//! - `detectors/evm/function/`  — Function-level detectors
//!
//! ## Usage
//!
//! Run standalone via `verazt scan` or as part of the full analysis
//! pipeline via `verazt analyze`.

pub mod cli;
pub mod detector;
pub mod detectors;
pub mod engine;
pub mod registry;

pub use detector::{Confidence, DetectionLevel, ScanDetector, Target};
pub use engine::{ScanConfig, ScanEngine, ScanReport};
pub use registry::{ScanRegistry, register_all_detectors};
