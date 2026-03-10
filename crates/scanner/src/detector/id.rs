//! Detector Identity
//!
//! `DetectorId` enumerates the scanner's built-in bug detectors.
//! Unlike the analysis crate's `TypeId`-based pass identity, detector IDs
//! are a closed enum because the scanner needs them for CLI filtering,
//! human-readable output, and stable serialization.

/// Unique identifier for each built-in bug detector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DetectorId {
    // ── AST-based detectors ──────────────────────────────────────
    CeiViolation,
    DeadCode,
    Reentrancy,
    UncheckedCall,
    UninitializedStorage,

    // ── GREP-based detectors ─────────────────────────────────────
    CentralizationRisk,
    ConstantStateVar,
    Delegatecall,
    Deprecated,
    FloatingPragma,
    LowLevelCall,
    MissingAccessControl,
    Shadowing,
    TimestampDependence,
    TxOrigin,
    Visibility,

    // ── SIR structural detectors ─────────────────────────────────
    SirMissingAccessControl,
    SirMissingModifies,
    SirUncheckedArithmetic,
    SirTxOriginAuth,
    SirAcquiresMismatch,
    SirMissingPdaConstraint,

    // ── AIR dataflow detectors ───────────────────────────────────
    AIRReentrancy,
    AIRAccessControl,
    AIRArithmetic,
}

impl DetectorId {
    /// Return a stable kebab-case string for CLI and serialization.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CeiViolation => "cei-violation",
            Self::DeadCode => "dead-code",
            Self::Reentrancy => "reentrancy",
            Self::UncheckedCall => "unchecked-call",
            Self::UninitializedStorage => "uninitialized-storage",
            Self::CentralizationRisk => "centralization-risk",
            Self::ConstantStateVar => "constant-state-var",
            Self::Delegatecall => "delegatecall",
            Self::Deprecated => "deprecated",
            Self::FloatingPragma => "floating-pragma",
            Self::LowLevelCall => "low-level-call",
            Self::MissingAccessControl => "missing-access-control",
            Self::Shadowing => "shadowing",
            Self::TimestampDependence => "timestamp-dependence",
            Self::TxOrigin => "tx-origin",
            Self::Visibility => "visibility",
            Self::SirMissingAccessControl => "sir-missing-access-control",
            Self::SirMissingModifies => "sir-missing-modifies",
            Self::SirUncheckedArithmetic => "sir-unchecked-arithmetic",
            Self::SirTxOriginAuth => "sir-tx-origin-auth",
            Self::SirAcquiresMismatch => "sir-acquires-mismatch",
            Self::SirMissingPdaConstraint => "sir-missing-pda-constraint",
            Self::AIRReentrancy => "air-reentrancy",
            Self::AIRAccessControl => "air-access-control",
            Self::AIRArithmetic => "air-arithmetic",
        }
    }
}

impl std::fmt::Display for DetectorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
