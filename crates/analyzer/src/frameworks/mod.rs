//! Reusable analysis infrastructure (IR-agnostic).
//!
//! - `bir/` — positional views of BIR functions and state accesses
//! - `cfa/`: control-flow analysis utilities (dominator trees over BIR CFGs)
//! - `dfa/`: dataflow analysis framework (lattices and the op solver)

pub mod bir;
pub mod cfa;
pub mod dfa;
