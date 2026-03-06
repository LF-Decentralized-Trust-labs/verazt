//! DFA-Based Bug Detectors
//!
//! Detectors that operate on the IR representation using data flow analysis.
//! These require CFG construction and various data flow analyses.

pub mod cei_violation;
pub mod dead_code;
pub mod reentrancy;
pub mod unchecked_call;
pub mod uninitialized;

// SCIR structural detectors
pub mod scir_missing_access_control;
pub mod scir_missing_modifies;
pub mod scir_unchecked_arithmetic;
pub mod scir_tx_origin_auth;
pub mod scir_acquires_mismatch;
pub mod scir_missing_pda_constraint;

// ANIR dataflow detectors
pub mod anir_reentrancy;
pub mod anir_access_control;
pub mod anir_arithmetic;

pub use cei_violation::CeiViolationDfaDetector;
pub use dead_code::DeadCodeDfaDetector;
pub use reentrancy::ReentrancyDfaDetector;
pub use unchecked_call::UncheckedCallDfaDetector;
pub use uninitialized::UninitializedDfaDetector;

// SCIR structural re-exports
pub use scir_missing_access_control::ScirMissingAccessControlDetector;
pub use scir_missing_modifies::ScirMissingModifiesDetector;
pub use scir_unchecked_arithmetic::ScirUncheckedArithmeticDetector;
pub use scir_tx_origin_auth::ScirTxOriginAuthDetector;
pub use scir_acquires_mismatch::ScirAcquiresMismatchDetector;
pub use scir_missing_pda_constraint::ScirMissingPdaConstraintDetector;

// ANIR dataflow re-exports
pub use anir_reentrancy::AnirReentrancyDetector;
pub use anir_access_control::AnirAccessControlDetector;
pub use anir_arithmetic::AnirArithmeticDetector;
