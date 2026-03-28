//! Detector Base Infrastructure
//!
//! Core types, traits, and registry for the detector framework.

pub mod id;
pub mod registry;
pub mod traits;

pub use id::DetectorId;
pub use registry::{DetectorRegistry, register_all_detectors};
pub use traits::{
    BugDetectionPass, ConfidenceLevel, DetectorError, DetectorResult, create_bug,
    create_bug_with_details,
};
