//! Dominance Analysis Pass
//!
//! Computes the dominator tree for each BIR function using the existing
//! `DomTree` infrastructure in `frameworks::cfa::domtree`, keyed by module
//! and function.

use crate::context::{AnalysisContext, ContextKey};
use crate::frameworks::cfa::domtree::DomTree;
use crate::passes::base::meta::{PassLevel, PassRepresentation};
use crate::passes::base::{AnalysisPass, Pass, PassResult};
use scirs::bir::cfg::FunctionId;
use std::any::TypeId;
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════
// Artifact
// ═══════════════════════════════════════════════════════════════════

/// Artifact key for dominance analysis.
///
/// Maps BIR module id → function → `DomTree`.
pub struct DominanceArtifact;

impl ContextKey for DominanceArtifact {
    type Value = HashMap<String, HashMap<FunctionId, DomTree>>;
    const NAME: &'static str = "dominance";
}

// ═══════════════════════════════════════════════════════════════════
// Pass
// ═══════════════════════════════════════════════════════════════════

/// Dominance analysis pass.
#[derive(Debug, Default)]
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
        PassRepresentation::Bir
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }
}

impl AnalysisPass for DominancePass {
    fn run(&self, ctx: &mut AnalysisContext) -> PassResult<()> {
        let result = ctx
            .bir_units()
            .iter()
            .map(|module| {
                let doms = module
                    .functions
                    .iter()
                    .filter_map(|func| Some((func.id.clone(), DomTree::build(func)?)))
                    .collect();
                (module.source_module_id.clone(), doms)
            })
            .collect();

        ctx.store::<DominanceArtifact>(result);
        ctx.mark_pass_completed(self.id());
        Ok(())
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
            Terminator::branch(OpRef(OpId(0)), BlockId(1), BlockId(2));
        let mut bb1 = BasicBlock::new(BlockId(1));
        bb1.term = Terminator::jump(BlockId(3));
        let mut bb2 = BasicBlock::new(BlockId(2));
        bb2.term = Terminator::jump(BlockId(3));
        let mut bb3 = BasicBlock::new(BlockId(3));
        bb3.term = Terminator::TxnExit { reverted: false };

        func.blocks = vec![bb0, bb1, bb2, bb3];

        let mut air_module = scirs::bir::Module::new("test".into());
        air_module.functions.push(func);

        let mut ctx = AnalysisContext::new(vec![], AnalysisConfig::default());
        ctx.set_bir_units(vec![air_module]);

        let pass = DominancePass;
        pass.run(&mut ctx).unwrap();

        let doms = ctx.get::<DominanceArtifact>().unwrap();
        let dom = &doms["test"][&FunctionId("test".into())];

        assert!(dom.dominates(BlockId(0), BlockId(1)));
        assert!(dom.dominates(BlockId(0), BlockId(3)));
        assert!(!dom.dominates(BlockId(1), BlockId(3)));
    }
}
