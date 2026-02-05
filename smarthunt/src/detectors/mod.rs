//! Detector framework for SmartHunt.
//!
//! This module provides the detector trait and registry for bug detection.

pub mod detector;
pub mod registry;
pub mod confidence;

// Detector implementations
pub mod tx_origin;
pub mod reentrancy;
pub mod unchecked_call;
pub mod floating_pragma;
pub mod low_level_call;
pub mod shadowing;
pub mod uninitialized;
pub mod deprecated;
pub mod visibility;
pub mod dead_code;
pub mod cei_violation;
pub mod missing_access_control;
pub mod delegatecall;
pub mod centralization_risk;
pub mod constant_state_var;
pub mod timestamp_dependence;

pub use detector::*;
pub use registry::*;
pub use confidence::*;
