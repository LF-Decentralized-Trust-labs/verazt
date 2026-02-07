//! AST-based Analysis Passes
//!
//! These passes operate on the source-level Abstract Syntax Tree (AST).

pub mod symbol_table;
pub mod type_index;
pub mod call_graph;
pub mod inheritance_graph;
pub mod modifier_analysis;

// Re-export pass implementations
pub use symbol_table::{SymbolTablePass, SymbolTable, FunctionId, SymbolTableExt};
pub use type_index::{TypeIndexPass, TypeIndex, TypeInfo, TypeIndexExt};
pub use call_graph::{CallGraphPass, CallGraph, CallSite, CallGraphExt};
pub use inheritance_graph::{InheritanceGraphPass, InheritanceGraph, InheritanceGraphExt};
pub use modifier_analysis::{ModifierAnalysisPass, ModifierAnalysis, ModifierInfo, ModifierAnalysisExt};
