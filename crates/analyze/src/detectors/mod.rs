//! Bug Detectors
//!
//! All detectors are grouped by bug category. Each file may contain multiple
//! detector variants (AST / SIR / ANIR) that target the same vulnerability
//! class.

// AST-based detectors (formerly "DFA" but perform manual AST walks)
pub mod cei_violation;
pub mod dead_code;
pub mod reentrancy;
pub mod unchecked_call;
pub mod uninitialized;

// GREP-based detectors (declarative AST pattern matching)
pub mod centralization_risk;
pub mod constant_state_var;
pub mod delegatecall;
pub mod deprecated;
pub mod floating_pragma;
pub mod low_level_call;
pub mod missing_access_control;
pub mod shadowing;
pub mod timestamp_dependence;
pub mod tx_origin;
pub mod visibility;

// SIR structural detectors
pub mod acquires_mismatch;
pub mod missing_modifies;
pub mod missing_pda_constraint;
pub mod sir_missing_access_control;
pub mod tx_origin_auth;
pub mod unchecked_arithmetic;

// ANIR dataflow detectors
pub mod anir_access_control;
pub mod anir_arithmetic;
pub mod anir_reentrancy;

// ── Re-exports ───────────────────────────────────────────────────────────

// AST-based detectors
pub use cei_violation::CeiViolationAstDetector;
pub use dead_code::DeadCodeAstDetector;
pub use reentrancy::ReentrancyAstDetector;
pub use unchecked_call::UncheckedCallAstDetector;
pub use uninitialized::UninitializedAstDetector;

// GREP-based detectors
pub use centralization_risk::CentralizationRiskGrepDetector;
pub use constant_state_var::ConstantStateVarGrepDetector;
pub use delegatecall::DelegatecallGrepDetector;
pub use deprecated::DeprecatedGrepDetector;
pub use floating_pragma::FloatingPragmaGrepDetector;
pub use low_level_call::LowLevelCallGrepDetector;
pub use missing_access_control::MissingAccessControlGrepDetector;
pub use shadowing::ShadowingGrepDetector;
pub use timestamp_dependence::TimestampDependenceGrepDetector;
pub use tx_origin::TxOriginGrepDetector;
pub use visibility::VisibilityGrepDetector;

// SIR structural detectors
pub use acquires_mismatch::SirAcquiresMismatchDetector;
pub use missing_modifies::SirMissingModifiesDetector;
pub use missing_pda_constraint::SirMissingPdaConstraintDetector;
pub use sir_missing_access_control::SirMissingAccessControlDetector;
pub use tx_origin_auth::SirTxOriginAuthDetector;
pub use unchecked_arithmetic::SirUncheckedArithmeticDetector;

// ANIR dataflow detectors
pub use anir_access_control::AnirAccessControlDetector;
pub use anir_arithmetic::AnirArithmeticDetector;
pub use anir_reentrancy::AnirReentrancyDetector;
