//! Def-Use Analysis Pass
//!
//! For each SSA `OpId`, computes the set of `OpId`s that reference its
//! result as an operand.  Single forward pass over all function blocks.

use crate::context::{AnalysisContext, ArtifactKey};
use crate::passes::base::meta::{PassLevel, PassRepresentation};
use crate::passes::base::{AnalysisPass, Pass, PassResult};
use scirs::air::cfg::Terminator;
use scirs::air::ops::{OpId, OpKind, OpRef};
use std::any::TypeId;
use std::collections::{HashMap, HashSet};

// ═══════════════════════════════════════════════════════════════════
// Artifact
// ═══════════════════════════════════════════════════════════════════

/// Artifact key for def-use analysis.
///
/// Maps `OpId` → set of `OpId`s that use this op's result.
pub struct DefUseArtifact;

impl ArtifactKey for DefUseArtifact {
    type Value = HashMap<OpId, HashSet<OpId>>;
    const NAME: &'static str = "def_use";
}

// ═══════════════════════════════════════════════════════════════════
// Pass
// ═══════════════════════════════════════════════════════════════════

/// Def-use analysis pass.
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
        PassRepresentation::Air
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }
}

impl AnalysisPass for DefUsePass {
    fn run(&self, ctx: &mut AnalysisContext) -> PassResult<()> {
        let mut result: HashMap<OpId, HashSet<OpId>> = HashMap::new();

        for module in ctx.air_units() {
            for func in &module.functions {
                // Ensure every defined op has an entry (possibly empty)
                for block in &func.blocks {
                    for op in &block.ops {
                        result.entry(op.id).or_default();
                    }
                }
                // Collect uses
                for block in &func.blocks {
                    for op in &block.ops {
                        let operands = collect_operands(&op.kind);
                        for OpRef(def_id) in operands {
                            result.entry(def_id).or_default().insert(op.id);
                        }
                    }
                    // Terminator operands
                    match &block.term {
                        Terminator::Branch { cond: OpRef(def_id), .. } => {
                            result.entry(*def_id).or_default();
                        }
                        _ => {}
                    }
                }
            }
        }

        ctx.store::<DefUseArtifact>(result);
        ctx.mark_pass_completed(self.id());
        Ok(())
    }

    fn is_completed(&self, ctx: &AnalysisContext) -> bool {
        ctx.is_pass_completed(self.id())
    }
}

/// Extract all OpRef operands from an OpKind.
fn collect_operands(kind: &OpKind) -> Vec<OpRef> {
    match kind {
        OpKind::BinOp { lhs, rhs, .. } => vec![*lhs, *rhs],
        OpKind::UnOp { operand, .. } => vec![*operand],
        OpKind::Phi(args) => args.iter().map(|(_, r)| *r).collect(),
        OpKind::Assert { cond } => vec![*cond],
        OpKind::Return(vals) => vals.clone(),
        OpKind::ExprStmt { expr } => vec![*expr],
        OpKind::Storage(s) => {
            let mut ops = vec![];
            if let Some(k) = s.key_operand {
                ops.push(k);
            }
            if let Some(v) = s.value_operand {
                ops.push(v);
            }
            ops
        }
        OpKind::Call(c) => c.args.clone(),
        OpKind::Const(_)
        | OpKind::Param { .. }
        | OpKind::TaintSrc(_)
        | OpKind::TaintSnk(_)
        | OpKind::PseudoValue { .. }
        | OpKind::Opaque { .. } => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::AnalysisConfig;
    use scirs::air::cfg::{BasicBlock, BlockId, Function, FunctionId, Terminator};
    use scirs::air::ops::{Op, OpId, OpKind, OpRef, SsaName};
    use scirs::sir::{BinOp, Lit, NumLit, OverflowSemantics, Type};

    #[test]
    fn test_def_use_basic() {
        let mut func = Function::new(FunctionId("test".into()), true);

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
        func.blocks = vec![bb0];

        let mut air_module = scirs::air::Module::new("test".into());
        air_module.functions.push(func);

        let mut ctx = AnalysisContext::new(vec![], AnalysisConfig::default());
        ctx.set_air_units(vec![air_module]);

        let pass = DefUsePass;
        pass.run(&mut ctx).unwrap();

        let du = ctx.get::<DefUseArtifact>().unwrap();

        // %0 is used by %2 (as lhs operand)
        assert!(du.get(&OpId(0)).unwrap().contains(&OpId(2)));
        // %1 is used by %2 (as rhs operand)
        assert!(du.get(&OpId(1)).unwrap().contains(&OpId(2)));
        // %2 is used by %3 (return)
        assert!(du.get(&OpId(2)).unwrap().contains(&OpId(3)));
        // %3 (return) has no users
        assert!(du.get(&OpId(3)).unwrap().is_empty());
    }
}
