//! AIR-layer analysis passes.

pub mod taint_propagation;

pub use taint_propagation::TaintPropagationPass;
