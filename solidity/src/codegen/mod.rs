//! Module for AST to IR generation (lowering).

pub mod ir_gen;

pub use ir_gen::{lower_source_unit, IrGen};
