//! Bug Detectors
//!
//! Detectors are organised into tiers:
//! - `base/`: Core traits, ID types, and registry
//! - `sir/`: SIR syntactic scan detectors (single-pass tree walk)
//! - `bir/`: BIR dataflow detectors (ICFG / taint), placeholder

pub mod base;
pub mod bir;
pub mod sir;

// Re-export base infrastructure for convenience
pub use base::{
    BugDetectionPass, ConfidenceLevel, DetectorError, DetectorId, DetectorRegistry,
    DetectorResult, create_bug, create_bug_with_details, register_all_detectors,
};
