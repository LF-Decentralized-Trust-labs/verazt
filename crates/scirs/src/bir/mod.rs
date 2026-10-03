//! BIR — Basic Block IR
//!
//! BIR is a graph-structured, SSA-form IR optimised for static dataflow
//! analysis and verification. Dialect constructs lower either to shared
//! feature ops (`Load`, `Store`, `Call`, `Env`, `Emit`) or to typed
//! per-chain ops (`Dialect`); generic analyses use the derived semantics in
//! `interfaces` (operands, storage accesses, taint sources/sinks, call risk).

pub mod alias;
pub mod call_graph;
pub mod cfg;
pub mod interfaces;
pub mod module;
pub mod ops;
pub mod pdg;
pub mod summary;
pub mod taint;
pub mod utils;
pub mod verifier;

// Re-exports for convenient access
pub use alias::*;
pub use call_graph::*;
pub use cfg::*;
pub use interfaces::*;
pub use module::*;
pub use ops::*;
pub use pdg::*;
pub use summary::*;
pub use taint::*;
