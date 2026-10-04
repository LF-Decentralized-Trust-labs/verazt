//! Def-Use Analysis Pass
//!
//! For each SSA `OpId`, computes the set of sites that reference its result
//! as an operand: ops, and block terminators (branch conditions and block
//! arguments). Single forward pass over all function blocks.

use crate::context::{AnalysisContext, ContextKey};
use crate::passes::base::meta::{PassLevel, PassRepresentation};
use crate::passes::base::{AnalysisPass, Pass, PassResult};
use scirs::bir::cfg::{BlockId, Function, FunctionId};
use scirs::bir::ops::{OpId, OpRef};
use std::any::TypeId;
use std::collections::{HashMap, HashSet};

// ═══════════════════════════════════════════════════════════════════
// Artifact
// ═══════════════════════════════════════════════════════════════════

/// Artifact key for def-use analysis.
///
/// Maps BIR module id → function → `OpId` → set of sites that use this
/// value. `OpId`s are only unique within a function.
pub struct DefUseArtifact;

impl ContextKey for DefUseArtifact {
    type Value = HashMap<String, HashMap<FunctionId, HashMap<OpId, HashSet<UseSite>>>>;
    const NAME: &'static str = "def_use";
}

/// A site that reads an SSA value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UseSite {
    /// An op, as an operand.
    Op(OpId),
    /// The terminator of a block, as a branch condition or block argument.
    Terminator(BlockId),
}

// ═══════════════════════════════════════════════════════════════════
// Pass
// ═══════════════════════════════════════════════════════════════════

/// Def-use analysis pass.
#[derive(Debug, Default)]
pub struct DefUsePass;

impl Pass for DefUsePass {
    fn name(&self) -> &'static str {
        "def-use"
    }

    fn description(&self) -> &'static str {
        "Compute def-use chains for SSA values"
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

impl AnalysisPass for DefUsePass {
    type Artifact = DefUseArtifact;

    fn run(
        &self,
        ctx: &AnalysisContext,
    ) -> PassResult<HashMap<String, HashMap<FunctionId, HashMap<OpId, HashSet<UseSite>>>>> {
        let result = ctx
            .bir_units()
            .iter()
            .map(|module| {
                let funcs = module
                    .functions
                    .iter()
                    .map(|func| (func.id.clone(), function_def_use(func)))
                    .collect();
                (module.source_module_id.clone(), funcs)
            })
            .collect();

        Ok(result)
    }
}

/// The use sites of every SSA value of `func`.
fn function_def_use(func: &Function) -> HashMap<OpId, HashSet<UseSite>> {
    let mut result: HashMap<OpId, HashSet<UseSite>> = HashMap::new();
    // Ensure every definition (block parameter or op) has an entry
    // (possibly empty)
    for block in &func.blocks {
        for param in &block.params {
            result.entry(param.id).or_default();
        }
        for op in &block.ops {
            result.entry(op.id).or_default();
        }
    }
    // Collect uses
    for block in &func.blocks {
        for op in &block.ops {
            for OpRef(def_id) in op.kind.operands() {
                result.entry(def_id).or_default().insert(UseSite::Op(op.id));
            }
        }
        for OpRef(def_id) in block.term.operands() {
            result
                .entry(def_id)
                .or_default()
                .insert(UseSite::Terminator(block.id));
        }
    }
    result
}

// ========================================================================
// Tests
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::AnalysisConfig;
    use scirs::bir::cfg::{BasicBlock, BlockCall, BlockParam, Terminator};
    use scirs::bir::ops::{Op, OpKind, SsaName};
    use scirs::sir::{BinOp, OverflowSemantics, Type};

    /// The def-use chains of the single function `f` made of `blocks`.
    fn analyze(blocks: Vec<BasicBlock>) -> HashMap<OpId, HashSet<UseSite>> {
        let mut func = Function::new(FunctionId("f".into()), true);
        func.blocks = blocks;
        let mut module = scirs::bir::Module::new("m".into());
        module.functions.push(func);
        let mut ctx = AnalysisContext::new(vec![], AnalysisConfig::default());
        ctx.set_bir_units(vec![module]);
        DefUsePass.run(&ctx).unwrap()["m"][&FunctionId("f".into())].clone()
    }

    #[test]
    fn test_terminator_operands_are_uses() {
        // bb0: %0 = param 0; branch %0, bb1, bb2
        // bb1: %1 = param 1; jump bb2(%1)
        // bb2(%2): exit
        let mut bb0 = BasicBlock::new(BlockId(0));
        bb0.ops = vec![Op::new(OpId(0), OpKind::Param { index: 0 })];
        bb0.term = Terminator::branch(OpRef(OpId(0)), BlockId(1), BlockId(2));
        let mut bb1 = BasicBlock::new(BlockId(1));
        bb1.ops = vec![Op::new(OpId(1), OpKind::Param { index: 1 })];
        bb1.term = Terminator::Jump(BlockCall { block: BlockId(2), args: vec![OpRef(OpId(1))] });
        let mut bb2 = BasicBlock::new(BlockId(2));
        bb2.params = vec![BlockParam { id: OpId(2), name: SsaName::new("x", 0), ty: Type::Si256 }];
        bb2.term = Terminator::TxnExit { reverted: false };

        let du = analyze(vec![bb0, bb1, bb2]);
        assert_eq!(du[&OpId(0)], HashSet::from([UseSite::Terminator(BlockId(0))]));
        assert_eq!(du[&OpId(1)], HashSet::from([UseSite::Terminator(BlockId(1))]));
        assert!(du[&OpId(2)].is_empty());
    }

    #[test]
    fn test_def_use_basic() {
        let mut bb0 = BasicBlock::new(BlockId(0));
        // %0 = param 0
        let op0 = Op::new(OpId(0), OpKind::Param { index: 0 })
            .with_result(SsaName::new("a", 0), Type::Si256);
        // %1 = param 1
        let op1 = Op::new(OpId(1), OpKind::Param { index: 1 })
            .with_result(SsaName::new("b", 0), Type::Si256);
        // %2 = binop add %0, %1
        let op2 = Op::new(
            OpId(2),
            OpKind::BinOp {
                op: BinOp::Add,
                lhs: OpRef(OpId(0)),
                rhs: OpRef(OpId(1)),
                overflow: OverflowSemantics::Checked,
            },
        )
        .with_result(SsaName::new("c", 0), Type::Si256);
        // return %2
        let op3 = Op::new(OpId(3), OpKind::Return(vec![OpRef(OpId(2))]));

        bb0.ops = vec![op0, op1, op2, op3];
        bb0.term = Terminator::TxnExit { reverted: false };

        let du = analyze(vec![bb0]);

        // %0 is used by %2 (as lhs operand)
        assert!(du.get(&OpId(0)).unwrap().contains(&UseSite::Op(OpId(2))));
        // %1 is used by %2 (as rhs operand)
        assert!(du.get(&OpId(1)).unwrap().contains(&UseSite::Op(OpId(2))));
        // %2 is used by %3 (return)
        assert!(du.get(&OpId(2)).unwrap().contains(&UseSite::Op(OpId(3))));
        // %3 (return) has no users
        assert!(du.get(&OpId(3)).unwrap().is_empty());
    }
}
