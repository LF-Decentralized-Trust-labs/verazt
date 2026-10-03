//! Step 3: ICFG, Alias Sets, and Taint Initialization
//!
//! Populates the ICFG, AliasMap, CallGraph, TaintGraph, and function
//! summaries from SSA-renamed functions, using the derived semantics of
//! each op (`bir::interfaces`).

use crate::bir::cfg::{EdgeKind, FunctionId, ICFGNode, Terminator};
use crate::bir::module::Module;
use crate::bir::ops::{CallTarget, OpKind};
use crate::bir::summary::FunctionSummary;

/// Build the ICFG, alias sets, and taint graph for the module.
pub fn build_icfg(module: &mut Module) {
    // Phase 1: Add TxnEntry/TxnExit for each public function
    for func in &module.functions {
        if func.is_public {
            let entry_id = module
                .icfg
                .add_node(ICFGNode::TxnEntry { func: func.id.clone() });
            let exit_ok_id = module
                .icfg
                .add_node(ICFGNode::TxnExit { func: func.id.clone(), reverted: false });
            let exit_rev_id = module
                .icfg
                .add_node(ICFGNode::TxnExit { func: func.id.clone(), reverted: true });

            // Add CFG edges from entry to exit
            module
                .icfg
                .add_edge(entry_id, exit_ok_id, EdgeKind::CfgEdge);
            module
                .icfg
                .add_edge(entry_id, exit_rev_id, EdgeKind::CfgEdge);
        }
    }

    // Phase 2: Derive ICFG nodes, alias sets, call graph, taint seeds and
    // sinks, and summaries from each op's derived semantics.
    for func in &module.functions {
        let mut summary = FunctionSummary::new(func.id.clone());

        for block in &func.blocks {
            for op in &block.ops {
                if let Some(access) = op.kind.storage_access() {
                    module.alias_sets.register(
                        access.alias_group_id(),
                        op.id,
                        access.is_write,
                        access.keys.first().copied(),
                    );
                    if access.is_write {
                        summary.modifies.push(access.storage_ref());
                    }
                }

                if let Some(label) = op.kind.taint_source() {
                    module.taint_graph.seed(op.id, label);
                }
                if let Some(category) = op.kind.sink_category() {
                    module.taint_graph.register_sink(op.id, category);
                }

                if let OpKind::Call(call) = &op.kind {
                    if call.may_reenter() {
                        let ext_node_id =
                            module.icfg.add_node(ICFGNode::ExternalCallNode { op: op.id });
                        let reentry_id = module
                            .icfg
                            .add_node(ICFGNode::ReentryPoint { func: func.id.clone() });
                        module.icfg.add_edge(ext_node_id, reentry_id, EdgeKind::ReentryEdge);
                        summary.reentrancy_safe = false;
                    } else {
                        module.icfg.add_node(ICFGNode::CallSite { op: op.id });
                    }
                    if call.transfers_value() {
                        summary.value_transfer = true;
                    }
                    match &call.target {
                        CallTarget::Internal(callee) => {
                            module.call_graph.add_static_edge(func.id.clone(), callee.clone());
                        }
                        CallTarget::External(_) => {
                            let dynamic = FunctionId("<dynamic>".to_string());
                            module.call_graph.add_dynamic_edge(op.id, dynamic);
                        }
                    }
                } else if op.result.is_some() || op.kind.storage_access().is_some() {
                    module.icfg.add_node(ICFGNode::StmtNode { op: op.id });
                }
            }
        }

        // Check if function may revert (has any TxnExit(reverted=true) terminator)
        for block in &func.blocks {
            if let Terminator::TxnExit { reverted: true } = block.term {
                summary.may_revert = true;
            }
        }

        module.summaries.push(summary);
    }
}
