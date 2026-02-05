//! AST-based Analysis Passes
//!
//! This module contains analysis passes that operate on the AST representation.

pub mod symbol_table;
pub mod type_index;
pub mod inheritance_graph;
pub mod call_graph;
pub mod modifier_analysis;

pub use symbol_table::SymbolTablePass;
pub use type_index::TypeIndexPass;
pub use inheritance_graph::InheritanceGraphPass;
pub use call_graph::CallGraphPass;
pub use modifier_analysis::ModifierAnalysisPass;
