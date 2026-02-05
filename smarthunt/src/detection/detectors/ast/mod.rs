//! AST-Based Detectors
//!
//! Detectors that operate solely on the AST representation.
//! These detect bugs from source-level patterns without requiring IR.

pub mod tx_origin;
pub mod floating_pragma;
pub mod visibility;
pub mod deprecated;
pub mod low_level_call;
pub mod unchecked_call;
pub mod shadowing;
pub mod timestamp_dependence;
pub mod delegatecall;

pub use tx_origin::TxOriginDetector;
pub use floating_pragma::FloatingPragmaDetector;
pub use visibility::VisibilityDetector;
pub use deprecated::DeprecatedDetector;
pub use low_level_call::LowLevelCallDetector;
pub use unchecked_call::UncheckedCallDetector;
pub use shadowing::ShadowingDetector;
pub use timestamp_dependence::TimestampDependenceDetector;
pub use delegatecall::DelegatecallDetector;
