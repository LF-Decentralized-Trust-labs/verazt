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
pub mod scir;

// ANIR dataflow detectors
pub mod anir;

pub use cei_violation::CeiViolationDfaDetector;
pub use dead_code::DeadCodeDfaDetector;
pub use reentrancy::ReentrancyDfaDetector;
pub use unchecked_call::UncheckedCallDfaDetector;
pub use uninitialized::UninitializedDfaDetector;

// SCIR structural re-exports
pub use scir::{
    ScirAcquiresMismatchDetector, ScirMissingAccessControlDetector, ScirMissingModifiesDetector,
    ScirMissingPdaConstraintDetector, ScirTxOriginAuthDetector, ScirUncheckedArithmeticDetector,
};

// ANIR dataflow re-exports
pub use anir::{AnirAccessControlDetector, AnirArithmeticDetector, AnirReentrancyDetector};
