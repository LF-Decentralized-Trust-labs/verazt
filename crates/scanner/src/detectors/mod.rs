//! Bug Detectors
//!
//! All detectors are grouped by bug category. Each file may contain multiple
//! detector variants (SIR / AIR) that target the same vulnerability class.

// SIR structural detectors (migrated from AST / GREP)
pub mod cei_violation;
pub mod centralization_risk;
pub mod constant_state_var;
pub mod dead_code;
pub mod delegatecall;
pub mod deprecated;
pub mod floating_pragma;
pub mod low_level_call;
pub mod missing_access_control;
pub mod reentrancy;
pub mod shadowing;
pub mod timestamp_dependence;
pub mod tx_origin;
pub mod unchecked_call;
pub mod uninitialized;
pub mod visibility;

// SIR structural detectors (multi-chain / dialect-specific)
pub mod acquires_mismatch;
pub mod missing_modifies;
pub mod missing_pda_constraint;
pub mod sir_missing_access_control;
pub mod tx_origin_auth;
pub mod unchecked_arithmetic;

// AIR dataflow detectors
pub mod access_control;
pub mod arithmetic;

// ── Re-exports ───────────────────────────────────────────────────────────

// SIR structural detectors (migrated)
pub use cei_violation::CeiViolationSirDetector;
pub use centralization_risk::CentralizationRiskSirDetector;
pub use constant_state_var::ConstantStateVarSirDetector;
pub use dead_code::DeadCodeSirDetector;
pub use delegatecall::DelegatecallSirDetector;
pub use deprecated::DeprecatedSirDetector;
pub use floating_pragma::FloatingPragmaSirDetector;
pub use low_level_call::LowLevelCallSirDetector;
pub use missing_access_control::MissingAccessControlSirDetector;
pub use reentrancy::ReentrancySirDetector;
pub use shadowing::ShadowingSirDetector;
pub use timestamp_dependence::TimestampDependenceSirDetector;
pub use tx_origin::TxOriginSirDetector;
pub use unchecked_call::UncheckedCallSirDetector;
pub use uninitialized::UninitializedSirDetector;
pub use visibility::VisibilitySirDetector;

// SIR structural detectors (multi-chain)
pub use acquires_mismatch::SirAcquiresMismatchDetector;
pub use missing_modifies::SirMissingModifiesDetector;
pub use missing_pda_constraint::SirMissingPdaConstraintDetector;
pub use sir_missing_access_control::SirMissingAccessControlDetector;
pub use tx_origin_auth::SirTxOriginAuthDetector;
pub use unchecked_arithmetic::SirUncheckedArithmeticDetector;

// AIR dataflow detectors
pub use access_control::AIRAccessControlDetector;
pub use arithmetic::AIRArithmeticDetector;
pub use reentrancy::AIRReentrancyDetector;
