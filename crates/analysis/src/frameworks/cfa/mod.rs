//! Control-Flow Analysis (CFA) Utilities
//!
//! A **utility collection** built on `petgraph`, providing higher-level
//! graph-analysis algorithms over the AIR's ICFG and per-function CFGs.
//!
//! Unlike `dfa/` (a parameterizable framework), each sub-module here is a
//! standalone algorithm producing an answer consumed by downstream passes
//! or by `frameworks/dfa/`.
//!
//! ## Sub-modules
//!
//! - [`domtree`] — dominator and post-dominator trees (Lengauer-Tarjan over
//!   `petgraph::DiGraph`)
//! - [`loops`] — natural-loop detection from the dominator tree
//! - [`reachability`] — BFS/DFS reachability queries over function CFGs
//! - [`callgraph`] — inter-procedural call graph built from SIR call sites

pub mod callgraph;
pub mod domtree;
pub mod loops;
pub mod reachability;
