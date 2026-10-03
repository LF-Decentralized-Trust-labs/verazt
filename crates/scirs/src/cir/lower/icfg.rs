//! Step 3: ICFG, Alias Sets, and Taint Initialization
//!
//! Builds the interprocedural control flow graph and populates the
//! AliasMap, CallGraph, TaintGraph, and function summaries from
//! SSA-numbered functions, using the derived semantics of each op
//! (`bir::interfaces`).
//!
//! ICFG shape: each function has an entry and two exits (normal and
//! reverted); each block starts with a `BlockEntry` node followed by one
//! node per op, chained by `CfgEdge`s. Internal calls link to the callee
//! with `CallEdge`/`ReturnEdge`. External calls that may re-enter link to
//! every public function's entry with `ReentryEdge`, and those functions'
//! normal exits return to the call's return site.

use crate::bir::cfg::{
    BlockId, BlockLoc, EdgeKind, Function, FunctionId, ICFG, ICFGNode, ICFGNodeId, OpLoc,
    Terminator,
};
use crate::bir::module::Module;
use crate::bir::ops::{CallOp, CallTarget, OpKind};
use crate::bir::summary::FunctionSummary;
use std::collections::HashMap;

/// Entry and exit nodes of one function.
struct FunctionNodes {
    entry: ICFGNodeId,
    exit_ok: ICFGNodeId,
    exit_reverted: ICFGNodeId,
}

/// Interprocedural targets shared by all functions of a module.
struct ModuleNodes {
    functions: HashMap<FunctionId, FunctionNodes>,
    /// Public functions: the possible re-entry points.
    public: Vec<FunctionId>,
}

/// Build the ICFG, alias sets, call graph, taint graph, and summaries for
/// the module.
pub fn build_icfg(module: &mut Module) {
    module.icfg = build_module_icfg(&module.functions);

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
                        summary.reentrancy_safe = false;
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
                }
            }

            if let Terminator::TxnExit { reverted: true } = block.term {
                summary.may_revert = true;
            }
        }

        module.summaries.push(summary);
    }
}

/// Build the ICFG of all functions in a module.
fn build_module_icfg(functions: &[Function]) -> ICFG {
    let mut icfg = ICFG::new();
    let nodes = ModuleNodes {
        functions: functions
            .iter()
            .map(|func| (func.id.clone(), add_function_nodes(&mut icfg, &func.id)))
            .collect(),
        public: functions.iter().filter(|f| f.is_public).map(|f| f.id.clone()).collect(),
    };
    for func in functions {
        add_function_flow(&mut icfg, func, &nodes);
    }
    icfg
}

fn add_function_nodes(icfg: &mut ICFG, func: &FunctionId) -> FunctionNodes {
    FunctionNodes {
        entry: icfg.add_node(ICFGNode::TxnEntry { func: func.clone() }),
        exit_ok: icfg.add_node(ICFGNode::TxnExit { func: func.clone(), reverted: false }),
        exit_reverted: icfg.add_node(ICFGNode::TxnExit { func: func.clone(), reverted: true }),
    }
}

/// Add the blocks, ops, and control flow of one function.
fn add_function_flow(icfg: &mut ICFG, func: &Function, nodes: &ModuleNodes) {
    let own = &nodes.functions[&func.id];
    let block_nodes: HashMap<BlockId, ICFGNodeId> = func
        .blocks
        .iter()
        .map(|block| {
            let loc = BlockLoc { func: func.id.clone(), block: block.id };
            (block.id, icfg.add_node(ICFGNode::BlockEntry(loc)))
        })
        .collect();
    if let Some(entry) = func.blocks.first() {
        icfg.add_edge(own.entry, block_nodes[&entry.id], EdgeKind::CfgEdge);
    }

    for block in &func.blocks {
        let mut last = block_nodes[&block.id];
        for op in &block.ops {
            let loc = OpLoc { func: func.id.clone(), op: op.id };
            last = if let OpKind::Call(call) = &op.kind {
                add_call_flow(icfg, last, loc, call, nodes)
            } else {
                let node = icfg.add_node(ICFGNode::StmtNode(loc));
                icfg.add_edge(last, node, EdgeKind::CfgEdge);
                node
            };
        }

        match &block.term {
            Terminator::TxnExit { reverted } => {
                let exit = if *reverted { own.exit_reverted } else { own.exit_ok };
                icfg.add_edge(last, exit, EdgeKind::CfgEdge);
            }
            Terminator::Branch { .. } | Terminator::Jump(_) => {
                for succ in block.term.successors() {
                    if let Some(&target) = block_nodes.get(&succ) {
                        icfg.add_edge(last, target, EdgeKind::CfgEdge);
                    }
                }
            }
            Terminator::Unreachable => {}
        }
    }
}

/// Add the nodes and edges of a call after `pred`, returning the return
/// site where intraprocedural flow resumes.
fn add_call_flow(
    icfg: &mut ICFG,
    pred: ICFGNodeId,
    loc: OpLoc,
    call: &CallOp,
    nodes: &ModuleNodes,
) -> ICFGNodeId {
    let return_site = ICFGNode::ReturnSite(loc.clone());
    match &call.target {
        CallTarget::Internal(callee) => {
            let site = icfg.add_node(ICFGNode::CallSite(loc));
            let ret = icfg.add_node(return_site);
            icfg.add_edge(pred, site, EdgeKind::CfgEdge);
            match nodes.functions.get(callee) {
                Some(callee) => {
                    icfg.add_edge(site, callee.entry, EdgeKind::CallEdge);
                    icfg.add_edge(callee.exit_ok, ret, EdgeKind::ReturnEdge);
                }
                // The callee is not in this module; step over the call.
                None => icfg.add_edge(site, ret, EdgeKind::CfgEdge),
            }
            ret
        }
        CallTarget::External(_) => {
            let site = icfg.add_node(ICFGNode::ExternalCallNode(loc));
            let ret = icfg.add_node(return_site);
            icfg.add_edge(pred, site, EdgeKind::CfgEdge);
            icfg.add_edge(site, ret, EdgeKind::CfgEdge);
            if call.may_reenter() {
                for public in &nodes.public {
                    let reentered = &nodes.functions[public];
                    icfg.add_edge(site, reentered.entry, EdgeKind::ReentryEdge);
                    icfg.add_edge(reentered.exit_ok, ret, EdgeKind::ReturnEdge);
                }
            }
            ret
        }
    }
}

// ========================================================================
// Tests
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cir::lower::cfg::{ContractScope, build_cfg};
    use crate::sir::evm::{EvmExpr, EvmLowLevelCall};
    use crate::sir::{
        CallArgs, CallExpr, DialectExpr, Expr, ExprStmt, Lit, Loc, Param, Stmt, StringLit, Type,
        VarExpr,
    };
    use std::collections::{HashSet, VecDeque};

    fn var(name: &str) -> Expr {
        Expr::Var(VarExpr::new(name.to_string(), Type::I256, None))
    }

    fn expr_stmt(expr: Expr) -> Stmt {
        Stmt::Expr(ExprStmt { expr, span: None })
    }

    /// Public `f(recipient)` runs `g(); recipient.call("");`, and internal
    /// `g()` is empty.
    fn module_with_calls() -> Module {
        let scope = ContractScope {
            contract: "C".to_string(),
            functions: HashSet::from(["f".to_string(), "g".to_string()]),
            storage_vars: HashSet::new(),
        };
        let call_g = Expr::FunctionCall(CallExpr {
            callee: Box::new(var("g")),
            args: CallArgs::Positional(vec![]),
            ty: Type::None,
            span: None,
        });
        let call_out = Expr::Dialect(DialectExpr::Evm(EvmExpr::LowLevelCall(EvmLowLevelCall {
            target: Box::new(var("recipient")),
            data: Box::new(Expr::Lit(Lit::String(StringLit::new(String::new(), None)))),
            value: None,
            gas: None,
            loc: Loc::default(),
        })));
        let f_body = vec![expr_stmt(call_g), expr_stmt(call_out)];
        let f_params = vec![Param::new("recipient".to_string(), Type::I256)];

        let mut module = Module::new("m".to_string());
        let mut f = Function::new(FunctionId("C.f".to_string()), true);
        f.blocks = build_cfg(&f_body, &f_params, &scope);
        let mut g = Function::new(FunctionId("C.g".to_string()), false);
        g.blocks = build_cfg(&[], &[], &scope);
        module.functions = vec![f, g];
        build_icfg(&mut module);
        module
    }

    fn node_id(icfg: &ICFG, node: &ICFGNode) -> ICFGNodeId {
        ICFGNodeId(icfg.nodes.iter().position(|n| n == node).unwrap())
    }

    fn has_edge(icfg: &ICFG, from: &ICFGNode, to: &ICFGNode, kind: EdgeKind) -> bool {
        let (from, to) = (node_id(icfg, from), node_id(icfg, to));
        icfg.edges.iter().any(|(f, t, k)| *f == from && *t == to && *k == kind)
    }

    #[test]
    fn test_icfg_links_calls_returns_and_reentry() {
        let module = module_with_calls();
        let icfg = &module.icfg;
        let f = FunctionId("C.f".to_string());
        let g = FunctionId("C.g".to_string());
        let entry = |func: &FunctionId| ICFGNode::TxnEntry { func: func.clone() };
        let exit_ok = |func: &FunctionId| ICFGNode::TxnExit { func: func.clone(), reverted: false };

        // The internal call enters `g`, and `g` returns to the return site.
        let call_site = icfg.nodes.iter().find(|n| matches!(n, ICFGNode::CallSite(_))).unwrap();
        let ICFGNode::CallSite(loc) = call_site else { unreachable!() };
        let return_site = ICFGNode::ReturnSite(loc.clone());
        assert!(has_edge(icfg, call_site, &entry(&g), EdgeKind::CallEdge));
        assert!(has_edge(icfg, &exit_ok(&g), &return_site, EdgeKind::ReturnEdge));

        // The external call may re-enter public `f`, but not internal `g`.
        let external = icfg
            .nodes
            .iter()
            .find(|n| matches!(n, ICFGNode::ExternalCallNode(_)))
            .unwrap();
        assert!(has_edge(icfg, external, &entry(&f), EdgeKind::ReentryEdge));
        assert!(!has_edge(icfg, external, &entry(&g), EdgeKind::ReentryEdge));

        // `f`'s normal exit is reachable from its entry.
        let (start, goal) = (node_id(icfg, &entry(&f)), node_id(icfg, &exit_ok(&f)));
        let mut seen = HashSet::from([start]);
        let mut queue = VecDeque::from([start]);
        while let Some(node) = queue.pop_front() {
            for (_, to, _) in icfg.edges.iter().filter(|(from, _, _)| *from == node) {
                if seen.insert(*to) {
                    queue.push_back(*to);
                }
            }
        }
        assert!(seen.contains(&goal));
    }
}
