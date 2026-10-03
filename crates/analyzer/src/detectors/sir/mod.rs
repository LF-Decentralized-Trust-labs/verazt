//! SIR Scan Detectors: syntactic pattern matching
//!
//! Fast, lightweight security checks that operate on the SIR
//! representation via a single-pass tree walk. No dataflow or
//! control-flow frameworks are used.
//!
//! The `ScanEngine` walks the SIR hierarchy (Module → Contract →
//! Function) exactly **once**, dispatching to detectors at each level.
//! Inside `verazt analyze`, each detector is wrapped as a
//! `BugDetectionPass` by `ScanDetectorAdapter`.
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
pub mod engine;
pub mod evm;
pub mod registry;

pub use adapter::ScanDetectorAdapter;
pub use detector::{Confidence, DetectionLevel, ScanDetector, Target};
pub use engine::{ScanConfig, ScanEngine, ScanReport};
pub use registry::ScanRegistry;
