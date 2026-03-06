//! ANIR — Analysis-oriented Normalized IR
//!
//! ANIR is a graph-structured, SSA-form IR optimised for static dataflow
//! analysis. Its key invariant: every dialect construct that reaches ANIR
//! implements at least one of `StorageOp`, `CallOp`, `TaintSource`, or
//! `TaintSink`.

pub mod alias;
pub mod call_graph;
pub mod cfg;
pub mod interfaces;
pub mod lower;
pub mod module;
pub mod ops;
pub mod pdg;
pub mod summary;
pub mod taint;

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
