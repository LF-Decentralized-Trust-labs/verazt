//! IR-based Analysis Passes
//!
//! These passes operate on the Intermediate Representation (IR) of Solidity
//! code. IR provides a lower-level, more structured view suitable for control
//! flow and data flow analysis.

// ANIR generation and analysis passes
pub mod access_control;
pub mod arithmetic;
pub mod generation;
pub mod taint;

// Re-export ANIR passes
pub use access_control::AnirAccessControlPass;
pub use arithmetic::AnirArithmeticPass;
pub use generation::AnirGenerationPass;
pub use taint::AnirTaintPropagationPass;
