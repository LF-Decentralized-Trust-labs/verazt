//! Detection Engines
//!
//! Pluggable engines that power the scanner's detectors:
//!
//! - `pattern`: AST/SIR structural pattern matching. Detectors implement a
//!   condition and the engine handles traversal and result collection.
//! - `datalog`: (future) Relational reasoning over program facts expressed
//!   as Datalog rules.
//!
//! Dataflow analysis is provided by the shared `analysis` crate
//! (`analysis::absint`) and is used directly by detectors that require
//! reasoning across basic blocks.

pub mod pattern;
pub mod datalog;
