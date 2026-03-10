//! Reusable analysis infrastructure (IR-agnostic).
//!
//! - `dfa/` — dataflow analysis framework (operates on AIR)
//! - `cfa/` — control-flow analysis utilities (operates on AIR ICFG)

pub mod cfa;
pub mod dfa;
