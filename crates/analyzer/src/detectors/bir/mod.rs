//! BIR Dataflow Detectors
//!
//! All detectors that operate on the BIR (Basic IR) representation,
//! using CFG / ICFG / taint / alias-set patterns.

pub mod reentrancy;

pub use reentrancy::ReentrancyFlowDetector;
