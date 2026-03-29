//! Bug Detectors
//!
//! Detectors are organised into tiers:
//! - `base/`: Core traits, ID types, and registry
//! - `sir/`: SIR structural detectors (tree-walking)
//! - `bir/`: BIR dataflow detectors (ICFG / taint) — placeholder

pub mod base;
pub mod bir;
pub mod sir;

// Re-export base infrastructure for convenience
pub use base::{
    BugDetectionPass, ConfidenceLevel, DetectorError, DetectorId, DetectorRegistry,
    DetectorResult, create_bug, create_bug_with_details, register_all_detectors,
};

// Re-export SIR structural detectors
pub use sir::ArithmeticOverflowSirDetector;
pub use sir::BadRandomnessSirDetector;
pub use sir::CeiViolationSirDetector;
pub use sir::CentralizationRiskSirDetector;
pub use sir::ConstantStateVarSirDetector;
pub use sir::DeadCodeSirDetector;
pub use sir::DelegatecallSirDetector;
pub use sir::DenialOfServiceSirDetector;
pub use sir::DeprecatedSirDetector;
pub use sir::FloatingPragmaSirDetector;
pub use sir::FrontRunningSirDetector;
pub use sir::LowLevelCallSirDetector;
pub use sir::MissingAccessControlSirDetector;
pub use sir::ReentrancySirDetector;
pub use sir::ShadowingSirDetector;
pub use sir::ShortAddressSirDetector;
pub use sir::TimestampDependenceSirDetector;
pub use sir::TxOriginSirDetector;
pub use sir::UncheckedCallSirDetector;
pub use sir::UninitializedSirDetector;
pub use sir::VisibilitySirDetector;
