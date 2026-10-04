//! Extended Taint Analysis Pass
//!
//! Propagates taint seeds along the module's taint-graph edges and SSA
//! operands to a fixpoint and stores the result as a typed `TaintArtifact` (set of taint
//! labels per `OpId`).
//!
//! Extended sources: TxOrigin, Timestamp, MsgValue, ExternalCallReturn.
//! Extended sinks:  branch conditions, storage writes, arithmetic operands.

use crate::context::{AnalysisContext, ContextKey};
use crate::passes::base::meta::{PassLevel, PassRepresentation};
use crate::passes::base::{AnalysisPass, Pass, PassResult};
use crate::passes::bir::icfg::ICFGPass;
use scirs::bir::interfaces::TaintLabel;
use scirs::bir::ops::{DialectOp, EvmOp, OpId, OpKind};
use std::any::TypeId;
use std::collections::{HashMap, HashSet};

// ═══════════════════════════════════════════════════════════════════
// Artifact
// ═══════════════════════════════════════════════════════════════════

/// Artifact key for extended taint analysis.
///
/// Maps `OpId` → set of `TaintLabel` that reach this op.
pub struct TaintArtifact;

impl ContextKey for TaintArtifact {
    type Value = HashMap<OpId, HashSet<TaintLabel>>;
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
    fn run(&self, ctx: &mut AnalysisContext) -> PassResult<()> {
        let mut taint_map: HashMap<OpId, HashSet<TaintLabel>> = HashMap::new();

        for module in ctx.bir_units() {
            // Phase 1: Seed taint sources from taint graph and ops
            for seed in &module.taint_graph.seeds {
                taint_map.entry(seed.op).or_default().insert(seed.label);
            }

            // Also seed from taint-source ops in functions
            for func in &module.functions {
                for block in &func.blocks {
                    for op in &block.ops {
                        if let Some(label) = op.kind.taint_source() {
                            taint_map.entry(op.id).or_default().insert(label);
                        }
                    }
                }
            }

            // Phase 2: Propagate through taint graph edges (fixed-point)
            let mut changed = true;
            let mut iteration = 0;
            const MAX_ITERATIONS: usize = 100;

            while changed && iteration < MAX_ITERATIONS {
                changed = false;
                iteration += 1;

                for &(src, dst) in &module.taint_graph.propagation {
                    if let Some(src_labels) = taint_map.get(&src).cloned() {
                        let entry = taint_map.entry(dst).or_default();
                        for label in src_labels {
                            if entry.insert(label) {
                                changed = true;
                            }
                        }
                    }
                }
            }

            // Phase 3: Also propagate through SSA def-use within functions
            changed = true;
            iteration = 0;
            while changed && iteration < MAX_ITERATIONS {
                changed = false;
                iteration += 1;

                for func in &module.functions {
                    for block in &func.blocks {
                        // Values computed from operands carry their labels
                        for op in &block.ops {
                            if propagates_taint(&op.kind) {
                                let sources: Vec<OpId> =
                                    op.kind.operands().iter().map(|r| r.0).collect();
                                changed |= merge_labels(&mut taint_map, &sources, op.id);
                            }
                        }

                        // Block parameters receive the labels of their arguments
                        for call in block.term.block_calls() {
                            let Some(target) = func.blocks.iter().find(|b| b.id == call.block)
                            else {
                                continue;
                            };
                            for (param, arg) in target.params.iter().zip(&call.args) {
                                changed |= merge_labels(&mut taint_map, &[arg.0], param.id);
                            }
                        }
                    }
                }
            }
        }

        ctx.store::<TaintArtifact>(taint_map);
        ctx.mark_pass_completed(self.id());
        Ok(())
    }
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
    use scirs::bir::ops::{EnvVar, Op, OpId, OpKind, SsaName};
    use scirs::sir::Type;

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

        // Run ICFGPass first (dependency)
        crate::passes::bir::icfg::ICFGPass.run(&mut ctx).unwrap();

        let pass = TaintPass;
        pass.run(&mut ctx).unwrap();

        let taint = ctx.get::<TaintArtifact>().unwrap();
        let labels = taint.get(&OpId(0)).unwrap();
        assert!(labels.contains(&TaintLabel::UserControlled));
    }
}
