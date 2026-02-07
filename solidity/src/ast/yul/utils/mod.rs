// FIXME: merge all modules here into ast/utils and delete this module

//! Utility traits for Yul AST.

pub mod fold;
pub mod map;
pub mod visit;

pub use fold::{YulFold, yul_fold_default};
pub use map::{YulMap, yul_map_default};
pub use visit::{YulVisit, yul_visit_default};
