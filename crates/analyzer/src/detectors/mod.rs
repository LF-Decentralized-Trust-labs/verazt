//! Bug Detectors
//!
//! Detectors are organised into three tiers:
//! - `base/`: Core traits, ID types, and registry
//! - `sir/`: SIR structural detectors (tree-walking)
//! - `bir/`: BIR dataflow detectors (ICFG / taint)

pub mod base;
pub mod bir;
pub mod sir;

// Re-export base infrastructure for convenience
pub use base::{
    BugDetectionPass, ConfidenceLevel, DetectorError, DetectorId, DetectorRegistry,
    DetectorResult, create_bug, create_bug_with_details, register_all_detectors,
};

// Re-export SIR structural detectors (migrated)
pub use sir::CeiViolationSirDetector;
pub use sir::CentralizationRiskSirDetector;
pub use sir::ConstantStateVarSirDetector;
pub use sir::DeadCodeSirDetector;
pub use sir::DelegatecallSirDetector;
pub use sir::DeprecatedSirDetector;
pub use sir::FloatingPragmaSirDetector;
pub use sir::LowLevelCallSirDetector;
pub use sir::MissingAccessControlSirDetector;
pub use sir::ReentrancySirDetector;
pub use sir::ShadowingSirDetector;
pub use sir::TimestampDependenceSirDetector;
pub use sir::TxOriginSirDetector;
pub use sir::UncheckedCallSirDetector;
pub use sir::UninitializedSirDetector;
pub use sir::VisibilitySirDetector;

// Re-export SIR structural detectors (multi-chain)
pub use sir::SirAcquiresMismatchDetector;
pub use sir::SirMissingAccessControlDetector;
pub use sir::SirMissingModifiesDetector;
pub use sir::SirMissingPdaConstraintDetector;
pub use sir::SirTxOriginAuthDetector;
pub use sir::SirUncheckedArithmeticDetector;

// Re-export BIR dataflow detectors
pub use bir::AIRAccessControlDetector;
pub use bir::AIRArithmeticDetector;
pub use bir::AIRReentrancyDetector;
