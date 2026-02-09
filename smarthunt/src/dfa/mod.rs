//! IR Data Flow Analysis Framework
//!
//! This module provides a generic framework for performing data flow analysis
//! on Solidity IR. It includes:
//!
//! - **CFG**: Enhanced control flow graph with precomputed def/use sets
//! - **Lattice**: Generic lattice framework for abstract domains
//! - **Solver**: Worklist-based data flow solver (forward and backward)
//! - **Analyses**: Built-in analyses (reaching definitions, liveness, etc.)
//!
//! # Example
//!
//! ```ignore
//! use smarthunt::dfa::{ControlFlowGraph, DataFlowSolver, Direction};
//! use smarthunt::dfa::lattice::PowerSetLattice;
//!
//! // Create a CFG from IR
//! let cfg = ControlFlowGraph::new("my_function".to_string(), entry_block);
//! cfg.compute_metadata();
//!
//! // Run forward analysis
//! let solver = DataFlowSolver::new(Direction::Forward, my_transfer);
//! let result = solver.solve(&cfg, initial_fact);
//! ```

pub mod cfg;
pub mod lattice;
pub mod solver;
pub mod utils;
pub mod var;

pub mod analyses;
pub mod detectors;

// Re-export commonly used types
pub use cfg::{BasicBlock, BasicBlockId, ControlFlowGraph, Terminator};
pub use lattice::{FlatLattice, Lattice, MapLattice, PowerSetLattice, ProductLattice};
pub use solver::{DataFlowResult, DataFlowSolver, Direction, Transfer};
pub use var::{VarId, VarScope};
