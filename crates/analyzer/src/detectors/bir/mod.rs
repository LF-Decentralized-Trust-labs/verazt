//! BIR Dataflow Detectors
//!
//! All detectors that operate on the BIR (Basic IR) representation,
//! using CFG / ICFG / taint / alias-set patterns.

pub mod cross_function_reentrancy;
pub mod reentrancy;
mod reentrant_sites;
#[cfg(test)]
mod test_fixtures;

pub use cross_function_reentrancy::CrossFunctionReentrancyDetector;
pub use reentrancy::ReentrancyFlowDetector;
