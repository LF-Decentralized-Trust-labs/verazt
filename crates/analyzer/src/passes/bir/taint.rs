//! Extended Taint Analysis Pass
//!
//! Propagates the taint sources of each function along SSA operands to a
//! fixpoint and stores the result as a typed `TaintArtifact` (set of taint
//! labels per `OpId`, keyed by module and function).
//!
//! Extended sources: TxOrigin, Timestamp, MsgValue, ExternalCallReturn.
//! Extended sinks:  branch conditions, storage writes, arithmetic operands.

use crate::context::{AnalysisContext, ContextKey};
use crate::passes::base::meta::{PassLevel, PassRepresentation};
use crate::passes::base::{AnalysisPass, Pass, PassResult};
use crate::passes::bir::icfg::ICFGPass;
use scirs::bir::cfg::{Function, FunctionId};
use scirs::bir::interfaces::TaintLabel;
use scirs::bir::ops::{DialectOp, EvmOp, OpId, OpKind};
use std::any::TypeId;
use std::collections::{HashMap, HashSet};

// ═══════════════════════════════════════════════════════════════════
// Artifact
// ═══════════════════════════════════════════════════════════════════

/// Artifact key for extended taint analysis.
///
/// Maps BIR module id → function → `OpId` → set of `TaintLabel` that reach
/// this op. `OpId`s are only unique within a function.
pub struct TaintArtifact;

impl ContextKey for TaintArtifact {
    type Value = HashMap<String, HashMap<FunctionId, HashMap<OpId, HashSet<TaintLabel>>>>;
    const NAME: &'static str = "taint";
}

// ═══════════════════════════════════════════════════════════════════
// Pass
// ═══════════════════════════════════════════════════════════════════

/// Extended taint analysis pass.
#[derive(Debug, Default)]
pub struct TaintPass;

impl Pass for TaintPass {
    fn name(&self) -> &'static str {
        "taint"
    }

    fn description(&self) -> &'static str {
        "Extended taint analysis with multiple label types"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Bir
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![TypeId::of::<ICFGPass>()]
    }
}

impl AnalysisPass for TaintPass {
    type Artifact = TaintArtifact;

    fn run(
        &self,
        ctx: &AnalysisContext,
    ) -> PassResult<HashMap<String, HashMap<FunctionId, HashMap<OpId, HashSet<TaintLabel>>>>> {
        let result = ctx
            .bir_units()
            .iter()
            .map(|module| {
                let funcs = module
                    .functions
                    .iter()
                    .map(|func| (func.id.clone(), function_taint(func)))
                    .collect();
                (module.source_module_id.clone(), funcs)
            })
            .collect();

        Ok(result)
    }
}

/// The taint labels reaching every SSA value of `func`: its taint-source
/// ops propagated along SSA operands and block arguments to a fixpoint.
fn function_taint(func: &Function) -> HashMap<OpId, HashSet<TaintLabel>> {
    let mut taint_map: HashMap<OpId, HashSet<TaintLabel>> = HashMap::new();
    for op in func.blocks.iter().flat_map(|block| &block.ops) {
        if let Some(label) = op.kind.taint_source() {
            taint_map.entry(op.id).or_default().insert(label);
        }
    }

    let mut changed = true;
    while changed {
        changed = false;
        for block in &func.blocks {
            // Values computed from operands carry their labels
            for op in &block.ops {
                if propagates_taint(&op.kind) {
                    let sources: Vec<OpId> = op.kind.operands().iter().map(|r| r.0).collect();
                    changed |= merge_labels(&mut taint_map, &sources, op.id);
                }
            }

            // Block parameters receive the labels of their arguments
            for call in block.term.block_calls() {
                let Some(target) = func.blocks.iter().find(|b| b.id == call.block) else {
                    continue;
                };
                for (param, arg) in target.params.iter().zip(&call.args) {
                    changed |= merge_labels(&mut taint_map, &[arg.0], param.id);
                }
            }
        }
    }
    taint_map
}

/// Returns `true` if the op's result is computed from its operands, so it
/// inherits their taint. Calls, loads, and effects introduce or consume
/// values instead.
fn propagates_taint(kind: &OpKind) -> bool {
    matches!(
        kind,
        OpKind::BinOp { .. }
            | OpKind::UnOp { .. }
            | OpKind::Opaque { .. }
            | OpKind::Dialect(DialectOp::Evm(EvmOp::Builtin(_)))
    )
}

/// Add the labels of all `sources` to `target`; returns `true` if any label
/// was new.
fn merge_labels(
    taint_map: &mut HashMap<OpId, HashSet<TaintLabel>>,
    sources: &[OpId],
    target: OpId,
) -> bool {
    let labels: HashSet<TaintLabel> =
        sources.iter().filter_map(|s| taint_map.get(s)).flatten().copied().collect();
    if labels.is_empty() {
        return false;
    }
    let entry = taint_map.entry(target).or_default();
    let before = entry.len();
    entry.extend(labels);
    entry.len() != before
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::AnalysisConfig;
    use scirs::bir::cfg::{BasicBlock, BlockId, Function, FunctionId, Terminator};
    use scirs::bir::ops::{EnvVar, Op, OpId, OpKind, OpRef, SsaName};
    use scirs::sir::{BoolLit, Lit, Type, UnOp};

    #[test]
    fn test_taint_pass_seed_propagation() {
        let mut func = Function::new(FunctionId("test".into()), true);
        let mut bb0 = BasicBlock::new(BlockId(0));

        // %0 = env Caller (msg.sender, a UserControlled source)
        let op0 = Op::new(OpId(0), OpKind::Env(EnvVar::Caller))
            .with_result(SsaName::new("sender", 0), Type::Si256);

        bb0.ops = vec![op0];
        bb0.term = Terminator::TxnExit { reverted: false };
        func.blocks = vec![bb0];

        let mut air_module = scirs::bir::Module::new("test".into());
        air_module.functions.push(func);

        let mut ctx = AnalysisContext::new(vec![], AnalysisConfig::default());
        ctx.set_bir_units(vec![air_module]);

        let taint = TaintPass.run(&ctx).unwrap();
        let labels = taint["test"][&FunctionId("test".into())].get(&OpId(0)).unwrap();
        assert!(labels.contains(&TaintLabel::UserControlled));
    }

    #[test]
    fn test_taint_does_not_leak_between_functions() {
        // f: %0 = env Caller. g: %0 = const, %1 = unop not %0.
        let mut f = Function::new(FunctionId("f".into()), true);
        let mut f_entry = BasicBlock::new(BlockId(0));
        f_entry.ops = vec![Op::new(OpId(0), OpKind::Env(EnvVar::Caller))];
        f_entry.term = Terminator::TxnExit { reverted: false };
        f.blocks = vec![f_entry];

        let mut g = Function::new(FunctionId("g".into()), true);
        let mut g_entry = BasicBlock::new(BlockId(0));
        let not = OpKind::UnOp { op: UnOp::Not, operand: OpRef(OpId(0)) };
        g_entry.ops = vec![
            Op::new(OpId(0), OpKind::Const(Lit::Bool(BoolLit::new(true, None)))),
            Op::new(OpId(1), not),
        ];
        g_entry.term = Terminator::TxnExit { reverted: false };
        g.blocks = vec![g_entry];

        let mut module = scirs::bir::Module::new("m".into());
        module.functions = vec![f, g];
        let mut ctx = AnalysisContext::new(vec![], AnalysisConfig::default());
        ctx.set_bir_units(vec![module]);

        let taint = TaintPass.run(&ctx).unwrap();
        let g_taint = &taint["m"][&FunctionId("g".into())];
        assert!(!g_taint.contains_key(&OpId(0)) && !g_taint.contains_key(&OpId(1)));
        assert!(taint["m"][&FunctionId("f".into())].contains_key(&OpId(0)));
    }
}
