//! Step 5: ICFG, Alias Sets, and Taint Initialization
//!
//! Populates the ICFG, AliasMap, and TaintGraph from SSA-renamed,
//! dialect-lowered functions.

use crate::cfg::{EdgeKind, FunctionId, ICFGNode};
use crate::interfaces::{CallOp, StorageOp, TaintLabel, TaintSource};
use crate::module::AnirModule;
use crate::ops::OpKind;
use crate::summary::FunctionSummary;

/// Build the ICFG, alias sets, and taint graph for the module.
pub fn build_icfg(module: &mut AnirModule) {
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

    // Phase 2: Process each op for interface tags
    for func in &module.functions {
        let mut summary = FunctionSummary::new(func.id.clone());

        for block in &func.blocks {
            for op in &block.ops {
                match &op.kind {
                    OpKind::Storage(storage_op) => {
                        // Register in alias map
                        module.alias_sets.register(
                            storage_op.alias_group_id(),
                            op.id,
                            storage_op.is_write(),
                            storage_op.key_operand,
                        );

                        // Track modifications in summary
                        if storage_op.is_write() {
                            summary.modifies.push(storage_op.to_storage_ref());
                        }

                        // Add ICFG node
                        module.icfg.add_node(ICFGNode::StmtNode { op: op.id });
                    }

                    OpKind::Call(call_op) => {
                        if call_op.call_risk().reentrancy {
                            let ext_node_id = module
                                .icfg
                                .add_node(ICFGNode::ExternalCallNode { op: op.id });
                            let reentry_id = module
                                .icfg
                                .add_node(ICFGNode::ReentryPoint { func: func.id.clone() });
                            module
                                .icfg
                                .add_edge(ext_node_id, reentry_id, EdgeKind::ReentryEdge);
                            summary.reentrancy_safe = false;
                        } else {
                            module.icfg.add_node(ICFGNode::CallSite { op: op.id });
                        }

                        if call_op.call_risk().value_transfer {
                            summary.value_transfer = true;
                        }

                        // Track in call graph
                        match &call_op.callee {
                            crate::interfaces::CallTarget::Static(name) => {
                                module
                                    .call_graph
                                    .add_static_edge(func.id.clone(), FunctionId(name.clone()));
                            }
                            crate::interfaces::CallTarget::Dynamic => {
                                module
                                    .call_graph
                                    .add_dynamic_edge(op.id, FunctionId("<dynamic>".to_string()));
                            }
                        }
                    }

                    OpKind::TaintSrc(taint_src) => {
                        // Seed the taint graph
                        module.taint_graph.seed(op.id, taint_src.taint_label());
                    }

                    OpKind::TaintSnk(taint_snk) => {
                        // Register taint sink
                        module
                            .taint_graph
                            .register_sink(op.id, taint_snk.sink_category());
                    }

                    OpKind::Return(_) => {
                        // Check for revert paths
                    }

                    _ => {
                        // Non-interface ops: add as StmtNode if they have a result
                        if op.result.is_some() {
                            module.icfg.add_node(ICFGNode::StmtNode { op: op.id });
                        }
                    }
                }
            }
        }

        // Check if function may revert (has any TxnExit(reverted=true) terminator)
        for block in &func.blocks {
            if let crate::cfg::Terminator::TxnExit { reverted: true } = block.term {
                summary.may_revert = true;
            }
        }

        module.summaries.push(summary);
    }

    // Phase 3: Taint propagation initialization
    // Initial edges: from each StorageOp read to its result (label StorageLoaded)
    // and from each SSA use of a tainted op
    for func in &module.functions {
        for block in &func.blocks {
            for op in &block.ops {
                if let OpKind::Storage(storage_op) = &op.kind {
                    if !storage_op.is_write {
                        module.taint_graph.seed(op.id, TaintLabel::StorageLoaded);
                    }
                }
            }
        }
    }
}
