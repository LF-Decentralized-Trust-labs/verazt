//! Module for AST to SCIR generation (lowering).

pub mod ir_gen;

pub use ir_gen::{IrGen, lower_source_unit};
