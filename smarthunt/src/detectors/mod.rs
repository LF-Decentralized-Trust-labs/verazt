//! Detector framework for SmartHunt.
//!
//! This module provides the detector trait and registry for bug detection.

pub mod detector;
pub mod registry;
pub mod confidence;
pub mod context; // Minimal stub for legacy support

// Remaining detector implementations (not yet migrated)
pub mod reentrancy;
pub mod uninitialized;
pub mod dead_code;
pub mod cei_violation;
pub mod missing_access_control;
pub mod centralization_risk;
pub mod constant_state_var;

pub use detector::*;
pub use registry::*;
pub use confidence::*;
pub use context::*;
