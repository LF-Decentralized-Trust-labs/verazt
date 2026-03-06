//! GREP-Based Bug Detectors
//!
//! Detectors that operate on the AST representation using declarative
//! pattern matching. These use the GREP framework for composable,
//! single-pass pattern detection.

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
