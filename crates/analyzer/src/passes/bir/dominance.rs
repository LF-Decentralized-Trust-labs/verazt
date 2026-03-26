//! Dominance Analysis Pass
//!
//! Computes the dominator tree for each BIR function using the existing
//! `DomTree` infrastructure in `frameworks::cfa::domtree`.  The result
//! is stored as a typed artifact mapping function name → `DomTree`.

use crate::context::{AnalysisContext, ContextKey};
use crate::frameworks::cfa::domtree::DomTree;
use crate::passes::base::meta::{PassLevel, PassRepresentation};
use crate::passes::base::{AnalysisPass, Pass, PassResult};
use std::any::TypeId;
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════
// Artifact
// ═══════════════════════════════════════════════════════════════════

/// Artifact key for dominance analysis.
///
/// Maps function name → `DomTree`.
pub struct DominanceArtifact;

impl ContextKey for DominanceArtifact {
    type Value = HashMap<String, DomTree>;
    const NAME: &'static str = "dominance";
}

// ═══════════════════════════════════════════════════════════════════
// Pass
// ═══════════════════════════════════════════════════════════════════

/// Dominance analysis pass.
pub struct DominancePass;

impl Pass for DominancePass {
    fn name(&self) -> &'static str {
        "dominance"
    }

    fn description(&self) -> &'static str {
        "Compute dominator trees for all BIR functions"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Air
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }
}

impl AnalysisPass for DominancePass {
    fn run(&self, ctx: &mut AnalysisContext) -> PassResult<()> {
        let mut result: HashMap<String, DomTree> = HashMap::new();

        for module in ctx.air_units() {
            for func in &module.functions {
                if let Some(dom) = DomTree::build(func) {
                    result.insert(func.id.0.clone(), dom);
                }
            }
        }

        ctx.store::<DominanceArtifact>(result);
        ctx.mark_pass_completed(self.id());
        Ok(())
    }

    fn is_completed(&self, ctx: &AnalysisContext) -> bool {
        ctx.is_pass_completed(self.id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::AnalysisConfig;
    use scirs::bir::cfg::{BasicBlock, BlockId, Function, FunctionId, Terminator};
    use scirs::bir::ops::{OpId, OpRef};

    #[test]
    fn test_dominance_pass() {
        // Build a diamond CFG function in an BIR module
        let mut func = Function::new(FunctionId("test".into()), true);

        let mut bb0 = BasicBlock::new(BlockId(0));
        bb0.term =
            Terminator::Branch { cond: OpRef(OpId(0)), then_bb: BlockId(1), else_bb: BlockId(2) };
        let mut bb1 = BasicBlock::new(BlockId(1));
        bb1.term = Terminator::Jump(BlockId(3));
        let mut bb2 = BasicBlock::new(BlockId(2));
        bb2.term = Terminator::Jump(BlockId(3));
        let mut bb3 = BasicBlock::new(BlockId(3));
        bb3.term = Terminator::TxnExit { reverted: false };

        func.blocks = vec![bb0, bb1, bb2, bb3];

        let mut air_module = scirs::bir::Module::new("test".into());
        air_module.functions.push(func);

        let mut ctx = AnalysisContext::new(vec![], AnalysisConfig::default());
        ctx.set_air_units(vec![air_module]);

        let pass = DominancePass;
        pass.run(&mut ctx).unwrap();

        let doms = ctx.get::<DominanceArtifact>().unwrap();
        let dom = doms.get("test").unwrap();

        assert!(dom.dominates(BlockId(0), BlockId(1)));
        assert!(dom.dominates(BlockId(0), BlockId(3)));
        assert!(!dom.dominates(BlockId(1), BlockId(3)));
    }
}
