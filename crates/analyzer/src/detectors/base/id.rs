//! Detector Identity
//!
//! `DetectorId` enumerates the scanner's built-in bug detectors.
//! Unlike the analysis crate's `TypeId`-based pass identity, detector IDs
//! are a closed enum because the scanner needs them for CLI filtering,
//! human-readable output, and stable serialization.

/// Unique identifier for each built-in bug detector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DetectorId {
    // ── SIR structural detectors ────────────────────────────────
    ArithmeticOverflow,
    BadRandomness,
    CeiViolation,
    CentralizationRisk,
    ConstantStateVar,
    DeadCode,
    Delegatecall,
    DenialOfService,
    Deprecated,
    FloatingPragma,
    FrontRunning,
    LowLevelCall,
    MissingAccessControl,
    Reentrancy,
    Shadowing,
    ShortAddress,
    TimestampDependence,
    TxOrigin,
    UncheckedCall,
    UninitializedStorage,
    Visibility,
}

impl DetectorId {
    /// Return a stable kebab-case string for CLI and serialization.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ArithmeticOverflow => "arithmetic-overflow",
            Self::BadRandomness => "bad-randomness",
            Self::CeiViolation => "cei-violation",
            Self::CentralizationRisk => "centralization-risk",
            Self::ConstantStateVar => "constant-state-var",
            Self::DeadCode => "dead-code",
            Self::Delegatecall => "delegatecall",
            Self::DenialOfService => "denial-of-service",
            Self::Deprecated => "deprecated",
            Self::FloatingPragma => "floating-pragma",
            Self::FrontRunning => "front-running",
            Self::LowLevelCall => "low-level-call",
            Self::MissingAccessControl => "missing-access-control",
            Self::Reentrancy => "reentrancy",
            Self::Shadowing => "shadowing",
            Self::ShortAddress => "short-address",
            Self::TimestampDependence => "timestamp-dependence",
            Self::TxOrigin => "tx-origin",
            Self::UncheckedCall => "unchecked-call",
            Self::UninitializedStorage => "uninitialized-storage",
            Self::Visibility => "visibility",
        }
    }
}

impl std::fmt::Display for DetectorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
