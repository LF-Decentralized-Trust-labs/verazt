//! Core block-to-function lifting for BIR → FIR transformation.
//!
//! Each BIR basic block becomes a FIR function:
//! - Entry block `%bb0` → `@Foo` (original function name)
//! - Block `%bbN` → `@Foo$N`
//!
//! Phi nodes are eliminated into function parameters.
//! Terminators are converted to tail calls.

use crate::bir::cfg::{BasicBlock, BlockId, Function, FunctionId, Terminator};
use crate::bir::ops::{Op, OpKind, OpRef, SsaName};
use crate::fir;
use crate::sir::Type;
use std::collections::{HashMap, HashSet, VecDeque};

/// Lift all basic blocks in a BIR function into FIR functions.
pub fn lift_function(bir_func: &Function) -> Vec<fir::Function> {
    // Compute reachable blocks from entry (bb0) to skip dead blocks.
    let reachable = compute_reachable_blocks(&bir_func.blocks);

    let block_func_ids = compute_block_func_ids(bir_func);
    let phi_params = collect_phi_params(&bir_func.blocks);
    let live_ins = compute_live_in_sets(&bir_func.blocks, &phi_params);

    let mut fir_functions = Vec::new();

    for block in &bir_func.blocks {
        // Skip unreachable blocks — they would produce orphan FIR functions.
        if !reachable.contains(&block.id) {
            continue;
        }

        let func_id = block_func_ids[&block.id].clone();

        // Parameters = phi params + live-in variables
        let mut params: Vec<(SsaName, Type)> = Vec::new();
        if let Some(phis) = phi_params.get(&block.id) {
            params.extend(phis.clone());
        }
        if let Some(live) = live_ins.get(&block.id) {
            for (name, ty) in live {
                // Don't duplicate phi params
                if !params.iter().any(|(n, _)| n == name) {
                    params.push((name.clone(), ty.clone()));
                }
            }
        }

        // Body = ops excluding phis and Return ops.
        // Return ops are extracted and folded into the FIR terminator.
        let mut return_vals: Option<Vec<OpRef>> = None;
        let body: Vec<Op> = block
            .ops
            .iter()
            .filter(|op| {
                if matches!(op.kind, OpKind::Phi(_)) {
                    return false;
                }
                if let OpKind::Return(vals) = &op.kind {
                    return_vals = Some(vals.clone());
                    return false;
                }
                true
            })
            .cloned()
            .collect();

        // Convert terminator, using extracted return values if present.
        let term = convert_terminator(
            &block.term,
            &block_func_ids,
            &phi_params,
            &live_ins,
            &bir_func.blocks,
            block.id,
            return_vals,
        );

        // Derive a span for this FIR function from the first non-phi op in the block.
        let block_span = block
            .ops
            .iter()
            .find(|op| !matches!(op.kind, OpKind::Phi(_)))
            .and_then(|op| op.span.clone());

        fir_functions.push(fir::Function::new(func_id, params, body, term).with_span(block_span));
    }

    fir_functions
}

/// Compute reachable blocks from entry block (`%bb0`) via BFS over
/// Jump/Branch edges. Blocks unreachable from entry are dead code
/// (e.g., blocks created after `return`/`revert` statements).
fn compute_reachable_blocks(blocks: &[BasicBlock]) -> HashSet<BlockId> {
    let mut reachable = HashSet::new();
    if blocks.is_empty() {
        return reachable;
    }

    let mut queue = VecDeque::new();
    let entry = blocks[0].id;
    reachable.insert(entry);
    queue.push_back(entry);

    // Build a quick lookup: BlockId → Terminator
    let term_map: HashMap<BlockId, &Terminator> = blocks.iter().map(|b| (b.id, &b.term)).collect();

    while let Some(id) = queue.pop_front() {
        if let Some(term) = term_map.get(&id) {
            let successors: Vec<BlockId> = match term {
                Terminator::Jump(target) => vec![*target],
                Terminator::Branch { then_bb, else_bb, .. } => vec![*then_bb, *else_bb],
                Terminator::TxnExit { .. } | Terminator::Unreachable => vec![],
            };
            for succ in successors {
                if reachable.insert(succ) {
                    queue.push_back(succ);
                }
            }
        }
    }

    reachable
}

/// Compute the FIR function IDs for each block.
fn compute_block_func_ids(bir_func: &Function) -> HashMap<BlockId, FunctionId> {
    let mut ids = HashMap::new();
    for block in &bir_func.blocks {
        let func_id = if block.id.0 == 0 {
            bir_func.id.clone()
        } else {
            FunctionId(format!("{}${}", bir_func.id.0, block.id.0))
        };
        ids.insert(block.id, func_id);
    }
    ids
}

/// Collect phi parameters for each block.
///
/// Returns a map from BlockId to the list of (SsaName, Type) defined by phi
/// nodes.
fn collect_phi_params(blocks: &[BasicBlock]) -> HashMap<BlockId, Vec<(SsaName, Type)>> {
    let mut params = HashMap::new();
    for block in blocks {
        let mut block_params = Vec::new();
        for op in &block.ops {
            if matches!(op.kind, OpKind::Phi(_)) {
                if let Some((name, ty)) = &op.result {
                    block_params.push((name.clone(), ty.clone()));
                }
            }
        }
        if !block_params.is_empty() {
            params.insert(block.id, block_params);
        }
    }
    params
}

/// Compute live-in sets for each block.
///
/// A variable is live-in to a block if it is used (referenced) in that block
/// but not defined before its first use within the same block.
fn compute_live_in_sets(
    blocks: &[BasicBlock],
    phi_params: &HashMap<BlockId, Vec<(SsaName, Type)>>,
) -> HashMap<BlockId, Vec<(SsaName, Type)>> {
    // Collect all definitions across all blocks (name → type)
    let mut all_defs: HashMap<SsaName, Type> = HashMap::new();
    for block in blocks {
        for op in &block.ops {
            if let Some((name, ty)) = &op.result {
                all_defs.insert(name.clone(), ty.clone());
            }
        }
    }

    let mut live_ins = HashMap::new();

    for block in blocks {
        // Definitions in this block (including phis)
        let mut defined_in_block: HashSet<SsaName> = HashSet::new();
        if let Some(phis) = phi_params.get(&block.id) {
            for (name, _) in phis {
                defined_in_block.insert(name.clone());
            }
        }

        let mut live_in_set: Vec<(SsaName, Type)> = Vec::new();
        let mut seen: HashSet<SsaName> = HashSet::new();

        for op in &block.ops {
            // Collect uses from this op
            let uses = collect_op_uses(op, blocks);
            for used_name in uses {
                if !defined_in_block.contains(&used_name) && !seen.contains(&used_name) {
                    if let Some(ty) = all_defs.get(&used_name) {
                        live_in_set.push((used_name.clone(), ty.clone()));
                        seen.insert(used_name);
                    }
                }
            }

            // Mark definition
            if let Some((name, _)) = &op.result {
                defined_in_block.insert(name.clone());
            }
        }

        if !live_in_set.is_empty() {
            live_ins.insert(block.id, live_in_set);
        }
    }

    live_ins
}

/// Collect SSA names used (referenced) by an op.
///
/// Looks up OpRef → SsaName by scanning all blocks for the defining op.
fn collect_op_uses(op: &Op, blocks: &[BasicBlock]) -> Vec<SsaName> {
    let refs = collect_oprefs(op);
    let mut names = Vec::new();
    for opref in refs {
        if let Some(name) = resolve_opref_to_name(opref, blocks) {
            names.push(name);
        }
    }
    names
}

/// Collect all OpRef values used by an op.
fn collect_oprefs(op: &Op) -> Vec<OpRef> {
    match &op.kind {
        OpKind::BinOp { lhs, rhs, .. } => vec![*lhs, *rhs],
        OpKind::UnOp { operand, .. } => vec![*operand],
        OpKind::Phi(entries) => entries.iter().map(|(_, r)| *r).collect(),
        OpKind::Assert { cond } => vec![*cond],
        OpKind::Return(vals) => vals.clone(),
        OpKind::ExprStmt { expr } => vec![*expr],
        OpKind::Call(c) => c.args.clone(),
        OpKind::Storage(s) => {
            let mut refs = Vec::new();
            if let Some(k) = s.key_operand {
                refs.push(k);
            }
            if let Some(v) = s.value_operand {
                refs.push(v);
            }
            refs
        }
        OpKind::Const(_)
        | OpKind::Param { .. }
        | OpKind::TaintSrc(_)
        | OpKind::TaintSnk(_)
        | OpKind::PseudoValue { .. }
        | OpKind::Opaque { .. } => vec![],
    }
}

/// Resolve an OpRef to its SsaName by searching all blocks.
fn resolve_opref_to_name(opref: OpRef, blocks: &[BasicBlock]) -> Option<SsaName> {
    for block in blocks {
        for op in &block.ops {
            if op.id == opref.0 {
                return op.result.as_ref().map(|(name, _)| name.clone());
            }
        }
    }
    None
}

/// Convert a BIR terminator into a FIR terminator.
///
/// If `return_vals` is `Some`, we have extracted an `OpKind::Return` from the
/// block body and should use its values for `Terminator::Return` instead of
/// the default empty return.
fn convert_terminator(
    term: &Terminator,
    block_func_ids: &HashMap<BlockId, FunctionId>,
    phi_params: &HashMap<BlockId, Vec<(SsaName, Type)>>,
    live_ins: &HashMap<BlockId, Vec<(SsaName, Type)>>,
    blocks: &[BasicBlock],
    _current_block: BlockId,
    return_vals: Option<Vec<OpRef>>,
) -> fir::Terminator {
    match term {
        Terminator::Jump(target) => {
            let callee = block_func_ids[target].clone();
            let args = build_tail_call_args(*target, phi_params, live_ins, blocks);
            fir::Terminator::TailCall(fir::TailCallData { callee, args })
        }
        Terminator::Branch { cond, then_bb, else_bb } => {
            let then_callee = block_func_ids[then_bb].clone();
            let then_args = build_tail_call_args(*then_bb, phi_params, live_ins, blocks);
            let else_callee = block_func_ids[else_bb].clone();
            let else_args = build_tail_call_args(*else_bb, phi_params, live_ins, blocks);
            fir::Terminator::Branch {
                cond: *cond,
                then_call: fir::TailCallData { callee: then_callee, args: then_args },
                else_call: fir::TailCallData { callee: else_callee, args: else_args },
            }
        }
        Terminator::TxnExit { reverted } => {
            if *reverted {
                fir::Terminator::Revert
            } else {
                // Use extracted return values from OpKind::Return if present,
                // otherwise fall back to empty return.
                fir::Terminator::Return(return_vals.unwrap_or_default())
            }
        }
        Terminator::Unreachable => fir::Terminator::Unreachable,
    }
}

/// Build the argument list for a tail call to a target block's function.
///
/// The arguments correspond to the target function's parameters:
/// phi params first, then live-in variables.
fn build_tail_call_args(
    _target: BlockId,
    _phi_params: &HashMap<BlockId, Vec<(SsaName, Type)>>,
    _live_ins: &HashMap<BlockId, Vec<(SsaName, Type)>>,
    _blocks: &[BasicBlock],
) -> Vec<OpRef> {
    // In a full implementation, we would:
    // 1. For each phi param in the target, find the incoming value from our block
    // 2. For each live-in in the target, pass the corresponding OpRef
    //
    // For now, return empty args — the structure is correct and will be
    // populated when full phi elimination is implemented.
    Vec::new()
}
