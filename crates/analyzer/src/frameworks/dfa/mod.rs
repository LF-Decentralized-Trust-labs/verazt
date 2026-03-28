//! Dataflow analysis framework.
//!
//! Generic lattice-based worklist solver and built-in analyses
//! (reaching defs, liveness, taint, etc.).

pub mod analyses;
pub mod annotated_cfg;
pub mod lattice;
pub mod solver;
pub mod utils;
pub mod var;

pub use annotated_cfg::{BasicBlock, BasicBlockId, ControlFlowGraph, Terminator};
pub use lattice::{FlatLattice, Lattice, MapLattice, PowerSetLattice, ProductLattice};
pub use solver::{DataFlowResult, DataFlowSolver, Direction, Transfer};
pub use var::{VarId, VarScope};
