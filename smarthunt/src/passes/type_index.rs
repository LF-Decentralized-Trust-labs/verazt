//! Type Index Pass.
//!
//! This pass builds the type index for fast type lookups.

use crate::engine::context::AnalysisContext;
use crate::graph::TypeIndex;
use crate::passes::{AnalysisPass, PassId, PassResult};

/// Pass for building the type index.
pub struct TypeIndexPass;

impl TypeIndexPass {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TypeIndexPass {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisPass for TypeIndexPass {
    fn id(&self) -> PassId {
        PassId::TypeIndex
    }

    fn name(&self) -> &'static str {
        "Type Index Construction"
    }

    fn description(&self) -> &'static str {
        "Build fast type lookup structures"
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::SymbolTable]
    }

    fn run(&self, context: &mut AnalysisContext) -> PassResult<()> {
        let type_index = TypeIndex::from_source_units(&context.source_units);
        context.type_index = Some(type_index);
        Ok(())
    }

    fn is_completed(&self, context: &AnalysisContext) -> bool {
        context.type_index.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_index_pass() {
        let pass = TypeIndexPass::new();
        assert_eq!(pass.id(), PassId::TypeIndex);
        assert_eq!(pass.dependencies(), vec![PassId::SymbolTable]);
    }
}
