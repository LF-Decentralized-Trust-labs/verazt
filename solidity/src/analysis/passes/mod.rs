//! Analysis Passes
//!
//! This module contains implementations of analysis passes that operate
//! on AST and IR representations.

pub mod ast;

// Re-export common passes
pub use ast::{SymbolTablePass, TypeIndexPass, InheritanceGraphPass, CallGraphPass, ModifierAnalysisPass};
