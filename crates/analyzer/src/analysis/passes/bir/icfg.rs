//! Interprocedural CFG (ICFG) Pass
//!
//! Extends the existing per-module ICFG with call/return edges and
//! external-call re-entry nodes.  The result augments the ICFG that
//! is already built by the BIR lowering step.

use crate::analysis::context::{AnalysisContext, ArtifactKey};
use crate::analysis::passes::base::meta::{PassLevel, PassRepresentation};
use crate::analysis::passes::base::{AnalysisPass, Pass, PassResult};
use scirs::bir::cfg::{EdgeKind, FunctionId, ICFG, ICFGNode};
use scirs::bir::interfaces::CallTarget;
use scirs::bir::ops::OpKind;
use std::any::TypeId;

// ═══════════════════════════════════════════════════════════════════
// Artifact
// ═══════════════════════════════════════════════════════════════════

/// Artifact key for the ICFG.
pub struct ICFGArtifact;

impl ArtifactKey for ICFGArtifact {
    type Value = Vec<ICFG>;
    const NAME: &'static str = "icfg";
}

// ═══════════════════════════════════════════════════════════════════
// Pass
// ═══════════════════════════════════════════════════════════════════

/// ICFG construction pass.
pub struct ICFGPass;

impl Pass for ICFGPass {
    fn name(&self) -> &'static str {
        "icfg"
    }

    fn description(&self) -> &'static str {
        "Build interprocedural control flow graphs"
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

impl AnalysisPass for ICFGPass {
    fn run(&self, ctx: &mut AnalysisContext) -> PassResult<()> {
        let mut icfgs: Vec<ICFG> = Vec::new();

        for module in ctx.air_units() {
            let mut icfg = module.icfg.clone();

            // Add transaction entry/exit nodes for each function
            for func in &module.functions {
                let entry_id = icfg.add_node(ICFGNode::TxnEntry { func: func.id.clone() });
                let exit_id =
                    icfg.add_node(ICFGNode::TxnExit { func: func.id.clone(), reverted: false });

                // Walk all ops to find call sites and external calls
                for block in &func.blocks {
                    for op in &block.ops {
                        if let OpKind::Call(call_op) = &op.kind {
                            let call_node_id = icfg.add_node(ICFGNode::CallSite { op: op.id });
                            let return_node_id = icfg.add_node(ICFGNode::ReturnSite { op: op.id });

                            match &call_op.callee {
                                CallTarget::Static(callee_name) => {
                                    // Internal call: add call/return edges
                                    let callee_entry = icfg.add_node(ICFGNode::TxnEntry {
                                        func: FunctionId(callee_name.clone()),
                                    });
                                    icfg.add_edge(call_node_id, callee_entry, EdgeKind::CallEdge);
                                    // Also link return back
                                    let callee_exit = icfg.add_node(ICFGNode::TxnExit {
                                        func: FunctionId(callee_name.clone()),
                                        reverted: false,
                                    });
                                    icfg.add_edge(
                                        callee_exit,
                                        return_node_id,
                                        EdgeKind::ReturnEdge,
                                    );
                                }
                                CallTarget::Dynamic => {
                                    // External call: tag as re-entry point
                                    let ext_node =
                                        icfg.add_node(ICFGNode::ExternalCallNode { op: op.id });
                                    icfg.add_edge(call_node_id, ext_node, EdgeKind::CallEdge);
                                    if call_op.call_risk.reentrancy {
                                        let reentry = icfg.add_node(ICFGNode::ReentryPoint {
                                            func: func.id.clone(),
                                        });
                                        icfg.add_edge(ext_node, reentry, EdgeKind::ReentryEdge);
                                    }
                                    icfg.add_edge(ext_node, return_node_id, EdgeKind::ReturnEdge);
                                }
                            }
                        }
                    }
                }

                let _ = (entry_id, exit_id); // nodes are stored in icfg
            }

            icfgs.push(icfg);
        }

        ctx.store::<ICFGArtifact>(icfgs);
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
    use crate::analysis::context::AnalysisConfig;

    #[test]
    fn test_icfg_pass_empty() {
        let mut ctx = AnalysisContext::new(vec![], AnalysisConfig::default());
        let pass = ICFGPass;
        pass.run(&mut ctx).unwrap();
        let icfgs = ctx.get::<ICFGArtifact>().unwrap();
        assert!(icfgs.is_empty());
    }
}
