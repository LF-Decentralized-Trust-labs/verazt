//! Detector Base Infrastructure
//!
//! Core types, traits, and registry for the detector framework.

pub mod id;
pub mod meta;
pub mod registry;
pub mod traits;

pub use id::DetectorId;
pub use meta::{DetectorMeta, Target};
pub use registry::{DetectorRegistry, register_all_detectors};
pub use traits::{BugDetectionPass, ConfidenceLevel, DetectorError, DetectorResult};
