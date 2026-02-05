//! Symbol Table Pass.
//!
//! This pass builds the symbol table for fast lookup of program entities.

use crate::engine::context::AnalysisContext;
use crate::graph::SymbolTable;
use crate::passes::{AnalysisPass, PassId, PassResult};

/// Pass for building the symbol table.
pub struct SymbolTablePass;

impl SymbolTablePass {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SymbolTablePass {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisPass for SymbolTablePass {
    fn id(&self) -> PassId {
        PassId::SymbolTable
    }

    fn name(&self) -> &'static str {
        "Symbol Table Extraction"
    }

    fn description(&self) -> &'static str {
        "Build fast lookup structures for program entities"
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![] // No dependencies - this is a foundation pass
    }

    fn run(&self, context: &mut AnalysisContext) -> PassResult<()> {
        let symbol_table = SymbolTable::from_source_units(&context.source_units);
        context.symbols = Some(symbol_table);
        Ok(())
    }

    fn is_completed(&self, context: &AnalysisContext) -> bool {
        context.symbols.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::config::Config;

    #[test]
    fn test_symbol_table_pass() {
        let pass = SymbolTablePass::new();
        assert_eq!(pass.id(), PassId::SymbolTable);
        assert!(pass.dependencies().is_empty());
    }
}
