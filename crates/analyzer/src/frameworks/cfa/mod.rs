//! Control-Flow Analysis (CFA) Utilities
//!
//! Graph-analysis algorithms over the per-function CFGs of the BIR, built on
//! `petgraph`.
//!
//! ## Sub-modules
//!
//! - [`domtree`]: dominator trees (Cooper-Harvey-Kennedy over
//!   `petgraph::DiGraph`)

pub mod domtree;
