//! SIR Structural Detectors
//!
//! All detectors that operate on the SIR (Structured IR) representation,
//! using tree-walking / structural pattern matching.

// SIR structural detectors (migrated from AST / GREP)
pub mod arithmetic_overflow;
pub mod bad_randomness;
pub mod cei_violation;
pub mod centralization_risk;
pub mod constant_state_var;
pub mod dead_code;
pub mod delegatecall;
pub mod denial_of_service;
pub mod deprecated;
pub mod floating_pragma;
pub mod front_running;
pub mod low_level_call;
pub mod missing_access_control;
pub mod reentrancy;
pub mod shadowing;
pub mod short_address;
pub mod timestamp_dependence;
pub mod tx_origin;
pub mod unchecked_call;
pub mod uninitialized;
pub mod visibility;

// Re-exports
pub use arithmetic_overflow::ArithmeticOverflowSirDetector;
pub use bad_randomness::BadRandomnessSirDetector;
pub use cei_violation::CeiViolationSirDetector;
pub use centralization_risk::CentralizationRiskSirDetector;
pub use constant_state_var::ConstantStateVarSirDetector;
pub use dead_code::DeadCodeSirDetector;
pub use delegatecall::DelegatecallSirDetector;
pub use denial_of_service::DenialOfServiceSirDetector;
pub use deprecated::DeprecatedSirDetector;
pub use floating_pragma::FloatingPragmaSirDetector;
pub use front_running::FrontRunningSirDetector;
pub use low_level_call::LowLevelCallSirDetector;
pub use missing_access_control::MissingAccessControlSirDetector;
pub use reentrancy::ReentrancySirDetector;
pub use shadowing::ShadowingSirDetector;
pub use short_address::ShortAddressSirDetector;
pub use timestamp_dependence::TimestampDependenceSirDetector;
pub use tx_origin::TxOriginSirDetector;
pub use unchecked_call::UncheckedCallSirDetector;
pub use uninitialized::UninitializedSirDetector;
pub use visibility::VisibilitySirDetector;
