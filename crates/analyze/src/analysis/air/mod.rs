//! IR-based Analysis Passes
//!
//! These passes operate on the Intermediate Representation (IR) of Solidity
//! code. IR provides a lower-level, more structured view suitable for control
//! flow and data flow analysis.

// AIR generation and analysis passes
pub mod access_control;
pub mod arithmetic;
pub mod generation;
pub mod taint;

// Re-export AIR passes
pub use access_control::AIRAccessControlPass;
pub use arithmetic::AIRArithmeticPass;
pub use generation::AIRGenerationPass;
pub use taint::AIRTaintPropagationPass;
