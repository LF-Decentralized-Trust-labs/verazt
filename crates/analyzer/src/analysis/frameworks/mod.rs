//! Reusable analysis infrastructure (IR-agnostic).
//!
//! - `dfa/` — dataflow analysis framework (operates on BIR)
//! - `cfa/` — control-flow analysis utilities (operates on BIR ICFG)
//! - `grep/` — AST/SIR structural pattern matching engine
//! - `datalog/` — relational reasoning over program facts

pub mod cfa;
pub mod datalog;
pub mod dfa;
pub mod grep;
