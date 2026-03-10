//! CFG Construction Pass
//!
//! This pass builds control flow graphs for all functions in the IR.
//! The underlying data types (`ControlFlowGraph`, `BasicBlock`, etc.)
//! live in `mlir::sir::cfg`.

use crate::context::AnalysisContext;
use crate::passes::base::{AnalysisPass, Pass, PassResult};
use std::any::TypeId;
use crate::passes::base::meta::{PassLevel, PassRepresentation};

pub use mlir::sir::cfg::{BasicBlock, BasicBlockId, ControlFlowGraph, Terminator};

/// CFG construction pass.
///
/// This pass builds control flow graphs for all functions in the IR.
pub struct CfgPass;

impl CfgPass {
    /// Create a new CFG construction pass.
    pub fn new() -> Self {
        Self
    }

    /// Build CFG for a single IR function.
    fn _build_cfg_for_function(
        &self,
        _func: &mlir::sir::FunctionDecl,
    ) -> PassResult<ControlFlowGraph> {
        // For now, create a simple single-block CFG
        // TODO: Implement full CFG construction from IR statements

        let entry_id = BasicBlockId(0);
        let mut cfg = ControlFlowGraph::new("function".to_string(), entry_id);

        let mut entry_block = BasicBlock::new(entry_id, Terminator::Return);
        entry_block.compute_successors();

        cfg.add_block(entry_block);
        cfg.compute_predecessors();

        Ok(cfg)
    }
}

impl Default for CfgPass {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for CfgPass {
    fn id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn name(&self) -> &'static str {
        "CFG Construction"
    }

    fn description(&self) -> &'static str {
        "Builds control flow graphs for IR functions"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Function
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ir
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }
}

impl AnalysisPass for CfgPass {
    fn run(&self, context: &mut AnalysisContext) -> PassResult<()> {
        // Check if IR is available
        if context.ir_units.is_none() {
            return Err(crate::passes::base::PassError::IrNotAvailable(self.name().to_string()));
        }

        // For now, just mark as completed
        // TODO: Iterate over IR functions and build CFGs

        // Store a placeholder artifact to indicate pass completed
        #[allow(deprecated)]
        context.store_artifact("cfg_constructed", true);

        Ok(())
    }

    fn is_completed(&self, context: &AnalysisContext) -> bool {
        #[allow(deprecated)]
        context.has_artifact("cfg_constructed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlir::sir::{BoolLit, Expr, Lit};

    #[test]
    fn test_basic_block_creation() {
        let id = BasicBlockId(0);
        let terminator = Terminator::Return;
        let block = BasicBlock::new(id, terminator);

        assert_eq!(block.id, id);
        assert_eq!(block.stmts.len(), 0);
        assert_eq!(block.successors.len(), 0);
    }

    #[test]
    fn test_basic_block_successors() {
        let mut block = BasicBlock::new(BasicBlockId(0), Terminator::Jump(BasicBlockId(1)));

        block.compute_successors();
        assert_eq!(block.successors, vec![BasicBlockId(1)]);
    }

    #[test]
    fn test_cfg_creation() {
        let entry = BasicBlockId(0);
        let cfg = ControlFlowGraph::new("test_function".to_string(), entry);

        assert_eq!(cfg.function_name, "test_function");
        assert_eq!(cfg.entry, entry);
        assert_eq!(cfg.blocks.len(), 0);
    }

    #[test]
    fn test_cfg_predecessors() {
        let mut cfg = ControlFlowGraph::new("test".to_string(), BasicBlockId(0));

        let mut block0 = BasicBlock::new(
            BasicBlockId(0),
            Terminator::Branch {
                condition: Expr::Lit(Lit::Bool(BoolLit { value: true, span: None })),
                true_block: BasicBlockId(1),
                false_block: BasicBlockId(2),
            },
        );
        block0.compute_successors();

        let mut block1 = BasicBlock::new(BasicBlockId(1), Terminator::Return);
        block1.compute_successors();

        let mut block2 = BasicBlock::new(BasicBlockId(2), Terminator::Return);
        block2.compute_successors();

        cfg.add_block(block0);
        cfg.add_block(block1);
        cfg.add_block(block2);
        cfg.compute_predecessors();

        let block1_preds = &cfg.get_block(BasicBlockId(1)).unwrap().predecessors;
        assert_eq!(block1_preds.len(), 1);
        assert!(block1_preds.contains(&BasicBlockId(0)));

        let block2_preds = &cfg.get_block(BasicBlockId(2)).unwrap().predecessors;
        assert_eq!(block2_preds.len(), 1);
        assert!(block2_preds.contains(&BasicBlockId(0)));
    }

    #[test]
    fn test_cfg_pass() {
        let pass = CfgPass::new();
        assert_eq!(pass.id(), TypeId::of::<CfgPass>());
        assert_eq!(pass.representation(), PassRepresentation::Ir);
    }
}
