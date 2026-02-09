//! Module for AST to IR generation (lowering).

pub mod ir_gen;

pub use ir_gen::{IrGen, lower_source_unit};
