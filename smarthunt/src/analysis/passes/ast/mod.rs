//! AST-based Analysis Passes
//!
//! These passes operate on the source-level Abstract Syntax Tree (AST).

pub mod call_graph;
pub mod inheritance_graph;
pub mod modifier_analysis;
pub mod symbol_table;
pub mod type_index;

// Re-export pass implementations
pub use call_graph::{CallGraph, CallGraphExt, CallGraphPass, CallSite};
pub use inheritance_graph::{InheritanceGraph, InheritanceGraphExt, InheritanceGraphPass};
pub use modifier_analysis::{
    ModifierAnalysis, ModifierAnalysisExt, ModifierAnalysisPass, ModifierInfo,
};
pub use symbol_table::{FunctionId, SymbolTable, SymbolTableExt, SymbolTablePass};
pub use type_index::{TypeIndex, TypeIndexExt, TypeIndexPass, TypeInfo};
