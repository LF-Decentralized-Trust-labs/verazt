//! Def-Use Analysis Pass
//!
//! For each SSA `OpId`, computes the set of `OpId`s that reference its
//! result as an operand.  Single forward pass over all function blocks.

use crate::context::{AnalysisContext, ContextKey};
use crate::passes::base::meta::{PassLevel, PassRepresentation};
use crate::passes::base::{AnalysisPass, Pass, PassResult};
use scirs::bir::cfg::{Function, FunctionId};
use scirs::bir::ops::{OpId, OpRef};
use std::any::TypeId;
use std::collections::{HashMap, HashSet};

// ═══════════════════════════════════════════════════════════════════
// Artifact
// ═══════════════════════════════════════════════════════════════════

/// Artifact key for def-use analysis.
///
/// Maps BIR module id → function → `OpId` → set of `OpId`s that use this
/// op's result. `OpId`s are only unique within a function.
pub struct DefUseArtifact;

impl ContextKey for DefUseArtifact {
    type Value = HashMap<String, HashMap<FunctionId, HashMap<OpId, HashSet<OpId>>>>;
    const NAME: &'static str = "def_use";
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
    ) -> PassResult<HashMap<String, HashMap<FunctionId, HashMap<OpId, HashSet<OpId>>>>> {
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

/// The users of every SSA value of `func`.
fn function_def_use(func: &Function) -> HashMap<OpId, HashSet<OpId>> {
    let mut result: HashMap<OpId, HashSet<OpId>> = HashMap::new();
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
                result.entry(def_id).or_default().insert(op.id);
            }
        }
        // Terminator operands (conditions and block arguments) have no op
        // id; just make sure their definitions appear
        for OpRef(def_id) in block.term.operands() {
            result.entry(def_id).or_default();
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::AnalysisConfig;
    use scirs::bir::cfg::{BasicBlock, BlockId, Function, FunctionId, Terminator};
    use scirs::bir::ops::{Op, OpId, OpKind, OpRef, SsaName};
    use scirs::sir::{BinOp, OverflowSemantics, Type};

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

        let mut air_module = scirs::bir::Module::new("test".into());
        air_module.functions.push(func);

        let mut ctx = AnalysisContext::new(vec![], AnalysisConfig::default());
        ctx.set_bir_units(vec![air_module]);

        let all = DefUsePass.run(&ctx).unwrap();
        let du = &all["test"][&FunctionId("test".into())];

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
