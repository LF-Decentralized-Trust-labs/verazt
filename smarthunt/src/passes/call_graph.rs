//! Call Graph Pass.
//!
//! This pass builds the call graph for inter-procedural analysis.

use crate::engine::context::AnalysisContext;
use crate::graph::{CallGraph, CallGraphBuilder};
use crate::passes::{AnalysisPass, PassId, PassResult};

/// Pass for building the call graph.
pub struct CallGraphPass;

impl CallGraphPass {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CallGraphPass {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisPass for CallGraphPass {
    fn id(&self) -> PassId {
        PassId::CallGraph
    }

    fn name(&self) -> &'static str {
        "Call Graph Construction"
    }

    fn description(&self) -> &'static str {
        "Build inter-procedural call relationships"
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::SymbolTable, PassId::Cfg]
    }

    fn run(&self, context: &mut AnalysisContext) -> PassResult<()> {
        let builder = CallGraphBuilder::new(&context.source_units);
        let call_graph = builder.build();
        context.call_graph = Some(call_graph);
        Ok(())
    }

    fn is_completed(&self, context: &AnalysisContext) -> bool {
        context.call_graph.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_graph_pass() {
        let pass = CallGraphPass::new();
        assert_eq!(pass.id(), PassId::CallGraph);
        assert_eq!(pass.dependencies(), vec![PassId::SymbolTable, PassId::Cfg]);
    }
}
