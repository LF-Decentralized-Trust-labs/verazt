//! Detector Module
//!
//! Provides the [`DetectorId`] enum and [`BugDetectionPass`] trait for
//! the scanner's bug detection framework.

pub mod id;
pub mod traits;

pub use id::DetectorId;
pub use traits::{
    BugDetectionPass, ConfidenceLevel, DetectorError, DetectorResult, create_bug,
    create_bug_with_details,
};
