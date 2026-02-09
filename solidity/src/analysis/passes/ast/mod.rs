//! AST-based Analysis Passes
//!
//! This module contains analysis passes that operate on the AST representation.

pub mod call_graph;
pub mod inheritance_graph;
pub mod modifier_analysis;
pub mod symbol_table;
pub mod type_index;

pub use call_graph::CallGraphPass;
pub use inheritance_graph::InheritanceGraphPass;
pub use modifier_analysis::ModifierAnalysisPass;
pub use symbol_table::SymbolTablePass;
pub use type_index::TypeIndexPass;
