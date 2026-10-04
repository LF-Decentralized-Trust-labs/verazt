//! Reusable analysis infrastructure (IR-agnostic).
//!
//! - `bir/` — positional views of BIR functions and state accesses
//! - `cfa/` — control-flow analysis utilities (operates on BIR ICFG)
//! - `dfa/` — dataflow analysis framework (lattices and solvers)

pub mod bir;
pub mod cfa;
pub mod dfa;
