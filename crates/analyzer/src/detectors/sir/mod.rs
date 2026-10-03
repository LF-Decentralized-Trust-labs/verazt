//! SIR Scan Detectors: syntactic pattern matching
//!
//! Fast, lightweight security checks that operate on the SIR
//! representation via a single-pass tree walk. No dataflow or
//! control-flow frameworks are used.
//!
//! Each detector is wrapped as a `BugDetectionPass` by
//! `ScanDetectorAdapter`, which walks the SIR hierarchy (Module →
//! Contract → Function) and dispatches to the detector at its level.
//!
//! Detectors are grouped by dialect and detection level:
//!
//! - `evm/module/`: EVM module-level detectors
//! - `evm/contract/`: EVM contract-level detectors
//! - `evm/function/`: EVM function-level detectors
//!
//! Future dialects add new sub-modules, e.g. `move/`.

pub mod adapter;
pub mod detector;
pub mod evm;
pub mod registry;

pub use adapter::ScanDetectorAdapter;
pub use detector::{Confidence, DetectionLevel, ScanDetector, Target};
pub use registry::ScanRegistry;
