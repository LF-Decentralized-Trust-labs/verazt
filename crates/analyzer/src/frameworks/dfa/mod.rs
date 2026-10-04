//! Dataflow analysis framework.
//!
//! A lattice abstraction and a worklist solver over the ops of BIR
//! functions.

pub mod lattice;
pub mod op_solver;

pub use lattice::{FlatLattice, Lattice, MapLattice, PowerSetLattice, ProductLattice};
pub use op_solver::{Direction, OpFacts};
