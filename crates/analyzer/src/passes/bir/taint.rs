//! Extended Taint Analysis Pass
//!
//! Builds on the existing `TaintPropagationPass` but stores the result
//! as a typed `TaintArtifact` (set of taint labels per `OpId`).
//!
//! Extended sources: TxOrigin, Timestamp, MsgValue, ExternalCallReturn.
//! Extended sinks:  branch conditions, storage writes, arithmetic operands.

use crate::context::{AnalysisContext, ContextKey};
use crate::passes::base::meta::{PassLevel, PassRepresentation};
use crate::passes::base::{AnalysisPass, Pass, PassResult};
use crate::passes::bir::icfg::ICFGPass;
use scirs::bir::interfaces::TaintLabel;
use scirs::bir::ops::{OpId, OpKind};
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
        PassRepresentation::Air
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![TypeId::of::<ICFGPass>()]
    }
}

impl AnalysisPass for TaintPass {
    fn run(&self, ctx: &mut AnalysisContext) -> PassResult<()> {
        let mut taint_map: HashMap<OpId, HashSet<TaintLabel>> = HashMap::new();

        for module in ctx.air_units() {
            // Phase 1: Seed taint sources from taint graph and ops
            for seed in &module.taint_graph.seeds {
                taint_map.entry(seed.op).or_default().insert(seed.label);
            }

            // Also seed from TaintSrc ops in functions
            for func in &module.functions {
                for block in &func.blocks {
                    for op in &block.ops {
                        if let OpKind::TaintSrc(src) = &op.kind {
                            taint_map.entry(op.id).or_default().insert(src.label);
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
                        for op in &block.ops {
                            // For BinOp: propagate labels from both operands
                            if let OpKind::BinOp { lhs, rhs, .. } = &op.kind {
                                let mut labels = HashSet::new();
                                if let Some(l) = taint_map.get(&lhs.0) {
                                    labels.extend(l.iter());
                                }
                                if let Some(r) = taint_map.get(&rhs.0) {
                                    labels.extend(r.iter());
                                }
                                if !labels.is_empty() {
                                    let entry = taint_map.entry(op.id).or_default();
                                    for label in labels {
                                        if entry.insert(label) {
                                            changed = true;
                                        }
                                    }
                                }
                            }

                            // For UnOp: propagate from operand
                            if let OpKind::UnOp { operand, .. } = &op.kind {
                                if let Some(labels) = taint_map.get(&operand.0).cloned() {
                                    let entry = taint_map.entry(op.id).or_default();
                                    for label in labels {
                                        if entry.insert(label) {
                                            changed = true;
                                        }
                                    }
                                }
                            }

                            // For Phi: union from all incoming
                            if let OpKind::Phi(args) = &op.kind {
                                let mut labels = HashSet::new();
                                for (_, incoming) in args {
                                    if let Some(l) = taint_map.get(&incoming.0) {
                                        labels.extend(l.iter());
                                    }
                                }
                                if !labels.is_empty() {
                                    let entry = taint_map.entry(op.id).or_default();
                                    for label in labels {
                                        if entry.insert(label) {
                                            changed = true;
                                        }
                                    }
                                }
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

    fn is_completed(&self, ctx: &AnalysisContext) -> bool {
        ctx.is_pass_completed(self.id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::AnalysisConfig;
    use scirs::bir::cfg::{BasicBlock, BlockId, Function, FunctionId, Terminator};
    use scirs::bir::ops::{Op, OpId, OpKind, OpRef, SsaName, TaintSourceOp};
    use scirs::sir::Type;

    #[test]
    fn test_taint_pass_seed_propagation() {
        let mut func = Function::new(FunctionId("test".into()), true);
        let mut bb0 = BasicBlock::new(BlockId(0));

        // %0 = taint source (UserControlled)
        let op0 = Op::new(
            OpId(0),
            OpKind::TaintSrc(TaintSourceOp {
                label: TaintLabel::UserControlled,
                dialect_name: "evm".into(),
                op_name: "msg_sender".into(),
            }),
        )
        .with_result(SsaName::new("sender", 0), Type::Si256);

        bb0.ops = vec![op0];
        bb0.term = Terminator::TxnExit { reverted: false };
        func.blocks = vec![bb0];

        let mut air_module = scirs::bir::Module::new("test".into());
        air_module.functions.push(func);

        let mut ctx = AnalysisContext::new(vec![], AnalysisConfig::default());
        ctx.set_air_units(vec![air_module]);

        // Run ICFGPass first (dependency)
        crate::passes::bir::icfg::ICFGPass.run(&mut ctx).unwrap();

        let pass = TaintPass;
        pass.run(&mut ctx).unwrap();

        let taint = ctx.get::<TaintArtifact>().unwrap();
        let labels = taint.get(&OpId(0)).unwrap();
        assert!(labels.contains(&TaintLabel::UserControlled));
    }
}
