//! Reusable analysis infrastructure (IR-agnostic).
//!
//! - `dfa/` — dataflow analysis framework (operates on BIR)
//! - `cfa/` — control-flow analysis utilities (operates on BIR ICFG)

pub mod cfa;
pub mod dfa;
