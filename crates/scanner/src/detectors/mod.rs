//! Scan Detectors
//!
//! Grouped by dialect and detection level:
//!
//! - `evm/module/`    — EVM module-level detectors
//! - `evm/contract/`  — EVM contract-level detectors
//! - `evm/function/`  — EVM function-level detectors
//!
//! Future dialects add new sub-modules:
//! - `move/module/`, `move/contract/`, `move/function/`

pub mod evm;

// Re-export all detector types for convenience
pub use evm::*;
