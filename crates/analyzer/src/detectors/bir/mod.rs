//! BIR Dataflow Detectors
//!
//! All detectors that operate on the BIR (Basic-block IR) representation,
//! using ICFG and dataflow analysis.

pub mod access_control;
pub mod arithmetic;
pub mod reentrancy;

pub use access_control::AIRAccessControlDetector;
pub use arithmetic::AIRArithmeticDetector;
pub use reentrancy::AIRReentrancyDetector;
