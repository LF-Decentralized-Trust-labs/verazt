//! Utility traits for SIR traversal and transformation.

pub mod export;
pub mod fold;
pub mod map;
pub mod printer;
pub mod query;
pub mod visit;

pub use fold::Fold;
pub use map::Map;
pub use printer::print_ir;
pub use visit::Visit;

pub use export::export_module;
pub use query::*;
