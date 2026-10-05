//! Step 1: CFG Construction and SSA
//!
//! Converts structured CIR statements into basic blocks with explicit
//! control flow edges (terminators), in SSA form with block parameters.
//! CIR already makes chain semantics explicit, so expressions map directly
//! onto BIR ops: `Load` / `Store`, resolved calls, `Env`, `Emit`, and the
//! typed dialect remainder.

use crate::bir::cfg::{BasicBlock, BlockId, BlockParam, FunctionId, Terminator};
use crate::bir::ops::{
    AnchorOp, AnchorPdaOp, CallOp, CallTarget, DialectOp, EmitOp, EvmBuiltinOp, EvmOp,
    ExternalCallee, LoadOp, MoveGlobalOp, MoveOp, MoveWriteRefOp, Op, OpId, OpKind, OpRef,
    Resource, SsaName, StoreOp,
};
use crate::cir::{
    CanonAnchorExpr, CanonDialectExpr, CanonDialectKind, CanonDialectStmt, CanonEvmExpr,
    CanonEvmStmt, CanonExpr, CanonMoveExpr, CanonMoveGlobalExpr, CanonMoveStmt, CanonParam,
    CanonResource, CanonStmt, CanonTryCatchStmt, Lit, Loc, OverflowSemantics, Type,
};
use crate::sir::BoolLit;
use std::collections::{HashMap, HashSet};

// ═══════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════

/// Name used for SSA values that do not correspond to a source variable.
const TMP_NAME: &str = "_tmp";

/// The function entry block.
const ENTRY: BlockId = BlockId(0);

/// Placeholder for a block argument whose value is not yet known; every
/// placeholder is overwritten when the parameter is filled.
const UNFILLED_ARG: OpRef = OpRef(OpId(usize::MAX));

/// The callee CIR uses to represent tuple construction and destructuring.
const TUPLE_CALLEE: &str = "__tuple__";

// ═══════════════════════════════════════════════════════════════════
// Data Structures
// ═══════════════════════════════════════════════════════════════════

/// Where `break` and `continue` jump inside a loop.
#[derive(Clone, Copy)]
struct LoopTargets {
    break_to: BlockId,
    continue_to: BlockId,
}

/// State for the CFG builder.
struct CfgBuilder<'c> {
    blocks: Vec<BasicBlock>,
    /// Contract name, used to qualify internal call targets.
    contract: &'c str,
    /// Current SSA definition of each local variable, per block.
    current_defs: HashMap<BlockId, HashMap<String, OpRef>>,
    /// Enclosing loops, innermost last.
    loops: Vec<LoopTargets>,
    next_block_id: usize,
    next_op_id: usize,
    /// Block parameters awaiting incoming arguments: (block, variable, index).
    pending_params: Vec<(BlockId, String, usize)>,
    /// Distinct predecessors of each block; `None` until the CFG is complete
    /// (all blocks are unsealed while statements are being flattened).
    preds: Option<HashMap<BlockId, Vec<BlockId>>>,
    /// `Symbol` ops (in the entry block) for names with no local definition.
    symbols: HashMap<String, OpRef>,
    /// Declared type of each local variable, used for block parameters.
    var_types: HashMap<String, Type>,
}

impl<'c> CfgBuilder<'c> {
    fn new(contract: &'c str) -> Self {
        CfgBuilder {
            blocks: Vec::new(),
            contract,
            current_defs: HashMap::new(),
            loops: Vec::new(),
            next_block_id: 0,
            next_op_id: 0,
            pending_params: Vec::new(),
            preds: None,
            symbols: HashMap::new(),
            var_types: HashMap::new(),
        }
    }

    fn new_block(&mut self) -> BlockId {
        let id = BlockId(self.next_block_id);
        self.next_block_id += 1;
        self.blocks.push(BasicBlock::new(id));
        id
    }

    fn new_op_id(&mut self) -> OpId {
        let id = OpId(self.next_op_id);
        self.next_op_id += 1;
        id
    }

    fn block_mut(&mut self, id: BlockId) -> &mut BasicBlock {
        &mut self.blocks[id.0]
    }

    fn append_op(&mut self, block: BlockId, op: Op) -> OpRef {
        let op_ref = OpRef(op.id);
        self.block_mut(block).ops.push(op);
        op_ref
    }

    fn set_terminator(&mut self, block: BlockId, term: Terminator) {
        self.block_mut(block).term = term;
    }

    /// Append a value-producing op named `name`.
    fn emit_value(
        &mut self,
        block: BlockId,
        kind: OpKind,
        name: &str,
        ty: Type,
        span: Option<&Loc>,
    ) -> OpRef {
        let op = Op::new(self.new_op_id(), kind).with_result(SsaName::new(name, 0), ty);
        self.append_op(block, attach_span(op, span))
    }

    /// Append an op executed only for its effect.
    fn emit_effect(&mut self, block: BlockId, kind: OpKind, span: Option<&Loc>) -> OpRef {
        let op = Op::new(self.new_op_id(), kind);
        self.append_op(block, attach_span(op, span))
    }

    fn declare_local(&mut self, name: &str, ty: Type) {
        self.var_types.insert(name.to_string(), ty);
    }
}

// ═══════════════════════════════════════════════════════════════════
// SSA construction
// ═══════════════════════════════════════════════════════════════════
//
// Braun et al., "Simple and Efficient Construction of Static Single
// Assignment Form" (CC 2013), with block parameters instead of phi nodes.
// Every block stays unsealed until the whole CFG is built: a read that is
// not defined locally creates a block parameter, whose incoming arguments
// are filled once all predecessors are known. Parameters that receive a
// single distinct value are then removed.

impl CfgBuilder<'_> {
    /// Record `value` as the current definition of `name` in `block`.
    fn write_variable(&mut self, block: BlockId, name: &str, value: OpRef) {
        self.current_defs
            .entry(block)
            .or_default()
            .insert(name.to_string(), value);
    }

    /// The SSA value of local variable `name` at the current end of `block`.
    fn read_variable(&mut self, block: BlockId, name: &str) -> OpRef {
        if let Some(value) = self
            .current_defs
            .get(&block)
            .and_then(|defs| defs.get(name))
        {
            return *value;
        }
        let preds = self
            .preds
            .as_ref()
            .map(|preds| preds.get(&block).cloned().unwrap_or_default());
        let value = match preds.as_deref() {
            None if block == ENTRY => self.symbol(name),
            None => {
                let (value, index) = self.add_block_param(block, name);
                self.pending_params.push((block, name.to_string(), index));
                value
            }
            Some([]) => self.symbol(name),
            Some([pred]) => self.read_variable(*pred, name),
            Some(_) => {
                let (value, index) = self.add_block_param(block, name);
                // Define before filling, so reads around a loop terminate.
                self.write_variable(block, name, value);
                self.fill_block_param(block, name, index);
                return value;
            }
        };
        self.write_variable(block, name, value);
        value
    }

    /// The `Symbol` op for a name with no local definition.
    fn symbol(&mut self, name: &str) -> OpRef {
        if let Some(symbol) = self.symbols.get(name) {
            return *symbol;
        }
        let kind = OpKind::Symbol { name: name.to_string() };
        let ty = self.var_types.get(name).cloned().unwrap_or(Type::None);
        let symbol = self.emit_value(ENTRY, kind, name, ty, None);
        self.symbols.insert(name.to_string(), symbol);
        symbol
    }

    /// Add a parameter for variable `name` to `block`, returning its value
    /// and index.
    fn add_block_param(&mut self, block: BlockId, name: &str) -> (OpRef, usize) {
        let id = self.new_op_id();
        let ty = self.var_types.get(name).cloned().unwrap_or(Type::None);
        let params = &mut self.block_mut(block).params;
        params.push(BlockParam { id, name: SsaName::new(name, 0), ty });
        let index = params.len() - 1;
        if self.preds.is_some() {
            self.reserve_arg_slots(block);
        }
        (OpRef(id), index)
    }

    /// Give every transfer into `block` one argument slot per parameter.
    fn reserve_arg_slots(&mut self, block: BlockId) {
        let arity = self.blocks[block.0].params.len();
        for pred in self.preds_of(block) {
            for call in self.blocks[pred.0].term.block_calls_mut() {
                if call.block == block {
                    call.args.resize(arity, UNFILLED_ARG);
                }
            }
        }
    }

    /// Pass the value of `name` from every predecessor as argument `index`.
    fn fill_block_param(&mut self, block: BlockId, name: &str, index: usize) {
        for pred in self.preds_of(block) {
            let value = self.read_variable(pred, name);
            for call in self.blocks[pred.0].term.block_calls_mut() {
                if call.block == block {
                    call.args[index] = value;
                }
            }
        }
    }

    fn preds_of(&self, block: BlockId) -> Vec<BlockId> {
        self.preds
            .as_ref()
            .and_then(|preds| preds.get(&block).cloned())
            .unwrap_or_default()
    }

    /// Mark the CFG complete and fill all pending block parameters.
    fn seal_blocks(&mut self) {
        let mut preds: HashMap<BlockId, Vec<BlockId>> = HashMap::new();
        for block in &self.blocks {
            for succ in block.term.successors() {
                let entry = preds.entry(succ).or_default();
                if !entry.contains(&block.id) {
                    entry.push(block.id);
                }
            }
        }
        self.preds = Some(preds);

        for index in 0..self.blocks.len() {
            self.reserve_arg_slots(BlockId(index));
        }
        for (block, name, index) in std::mem::take(&mut self.pending_params) {
            self.fill_block_param(block, &name, index);
        }
    }

    /// Remove parameters whose incoming arguments are all the same value
    /// (ignoring the parameter itself), and rewrite their uses.
    fn remove_trivial_params(&mut self) {
        let mut replacements: HashMap<OpRef, OpRef> = HashMap::new();
        let mut changed = true;
        while changed {
            changed = false;
            for block_index in 0..self.blocks.len() {
                let block = BlockId(block_index);
                for index in (0..self.blocks[block_index].params.len()).rev() {
                    let param = OpRef(self.blocks[block_index].params[index].id);
                    let incoming: HashSet<OpRef> = self
                        .incoming_args(block, index)
                        .into_iter()
                        .map(|arg| resolve(&replacements, arg))
                        .filter(|arg| *arg != param)
                        .collect();
                    if let [value] = incoming.into_iter().collect::<Vec<_>>()[..] {
                        replacements.insert(param, value);
                        self.remove_block_param(block, index);
                        changed = true;
                    }
                }
            }
        }

        for block in &mut self.blocks {
            for op in &mut block.ops {
                for operand in op.kind.operands_mut() {
                    *operand = resolve(&replacements, *operand);
                }
            }
            for operand in block.term.operands_mut() {
                *operand = resolve(&replacements, *operand);
            }
        }
    }

    /// Argument `index` of every transfer into `block`.
    fn incoming_args(&self, block: BlockId, index: usize) -> Vec<OpRef> {
        self.preds_of(block)
            .into_iter()
            .flat_map(|pred| self.blocks[pred.0].term.block_calls())
            .filter(|call| call.block == block)
            .map(|call| call.args[index])
            .collect()
    }

    fn remove_block_param(&mut self, block: BlockId, index: usize) {
        self.blocks[block.0].params.remove(index);
        for pred in self.preds_of(block) {
            for call in self.blocks[pred.0].term.block_calls_mut() {
                if call.block == block {
                    call.args.remove(index);
                }
            }
        }
    }
}

/// Follow replacement chains to the final value.
fn resolve(replacements: &HashMap<OpRef, OpRef>, mut value: OpRef) -> OpRef {
    while let Some(next) = replacements.get(&value) {
        value = *next;
    }
    value
}

// ═══════════════════════════════════════════════════════════════════
// CFG construction
// ═══════════════════════════════════════════════════════════════════

/// Build a CFG in SSA form from a function body of contract `contract`.
pub fn build_cfg(stmts: &[CanonStmt], params: &[CanonParam], contract: &str) -> Vec<BasicBlock> {
    let mut builder = CfgBuilder::new(contract);
    let entry = builder.new_block();

    // Create parameter ops
    for (i, param) in params.iter().enumerate() {
        builder.declare_local(&param.name, param.ty.clone());
        let kind = OpKind::Param { index: i };
        let value = builder.emit_value(entry, kind, &param.name, param.ty.clone(), None);
        builder.write_variable(entry, &param.name, value);
    }

    // Flatten the statement list into basic blocks
    let exit = flatten_stmts(&mut builder, stmts, entry);

    // If the exit block doesn't have a terminator, add TxnExit
    if builder.block_mut(exit).term == Terminator::Unreachable {
        builder.set_terminator(exit, Terminator::TxnExit { reverted: false });
    }

    // Complete SSA form now that every block's predecessors are known
    builder.seal_blocks();
    builder.remove_trivial_params();

    builder.blocks
}

/// Flatten a list of statements into basic blocks, returning the exit block.
fn flatten_stmts(builder: &mut CfgBuilder, stmts: &[CanonStmt], mut current: BlockId) -> BlockId {
    for stmt in stmts {
        current = flatten_stmt(builder, stmt, current);
    }
    current
}

/// Flatten a single statement, returning the block to continue from.
fn flatten_stmt(builder: &mut CfgBuilder, stmt: &CanonStmt, current: BlockId) -> BlockId {
    match stmt {
        CanonStmt::LocalVar(local_var) => {
            let span = local_var.span.as_ref();
            let decls: Vec<_> = local_var.vars.iter().collect();
            for decl in decls.iter().copied().flatten() {
                builder.declare_local(&decl.name, decl.ty.clone());
            }
            match (decls.as_slice(), &local_var.init) {
                // `T x = init;`
                ([Some(decl)], Some(init)) => {
                    let value =
                        lower_expr_named(builder, current, init, &decl.name, decl.ty.clone());
                    builder.write_variable(current, &decl.name, value);
                }
                // `(T a, , T c) = init;`
                (_, Some(init)) => {
                    let value = lower_expr(builder, current, init);
                    for (index, decl) in decls.iter().enumerate() {
                        let Some(decl) = decl else { continue };
                        let part = OpKind::Opaque {
                            description: format!("tuple_get {index}"),
                            operands: vec![value],
                        };
                        let part =
                            builder.emit_value(current, part, &decl.name, decl.ty.clone(), span);
                        builder.write_variable(current, &decl.name, part);
                    }
                }
                // `T x;` declares default-initialized variables.
                (_, None) => {
                    for decl in decls.iter().copied().flatten() {
                        let kind = OpKind::Const(Lit::Bool(BoolLit::new(false, None)));
                        let value =
                            builder.emit_value(current, kind, &decl.name, decl.ty.clone(), span);
                        builder.write_variable(current, &decl.name, value);
                    }
                }
            }
            current
        }

        CanonStmt::Assign(assign) => {
            let name = expr_name(&assign.lhs);
            let value = lower_expr_named(builder, current, &assign.rhs, &name, assign.rhs.typ());
            assign_local(builder, current, &assign.lhs, value, assign.span.as_ref());
            current
        }

        CanonStmt::AugAssign(aug) => {
            let lhs_ref = lower_expr(builder, current, &aug.lhs);
            let rhs_ref = lower_expr(builder, current, &aug.rhs);
            let kind = OpKind::BinOp {
                op: aug.op,
                lhs: lhs_ref,
                rhs: rhs_ref,
                overflow: OverflowSemantics::Checked,
            };
            let span = aug.span.as_ref();
            let value =
                builder.emit_value(current, kind, &expr_name(&aug.lhs), aug.lhs.typ(), span);
            assign_local(builder, current, &aug.lhs, value, span);
            current
        }

        CanonStmt::Expr(expr_stmt) => {
            let expr_ref = lower_expr(builder, current, &expr_stmt.expr);
            let kind = OpKind::ExprStmt { expr: expr_ref };
            builder.emit_effect(current, kind, expr_stmt.span.as_ref());
            current
        }

        CanonStmt::If(if_stmt) => {
            let cond_ref = lower_expr(builder, current, &if_stmt.cond);

            let then_block = builder.new_block();
            let merge_block = builder.new_block();

            let then_exit = flatten_stmts(builder, &if_stmt.then_body, then_block);
            if builder.block_mut(then_exit).term == Terminator::Unreachable {
                builder.set_terminator(then_exit, Terminator::jump(merge_block));
            }

            let else_target = match &if_stmt.else_body {
                Some(else_body) => {
                    let else_block = builder.new_block();
                    let else_exit = flatten_stmts(builder, else_body, else_block);
                    if builder.block_mut(else_exit).term == Terminator::Unreachable {
                        builder.set_terminator(else_exit, Terminator::jump(merge_block));
                    }
                    else_block
                }
                None => merge_block,
            };
            builder.set_terminator(current, Terminator::branch(cond_ref, then_block, else_target));

            merge_block
        }

        CanonStmt::While(while_stmt) => {
            let header = builder.new_block();
            let body_block = builder.new_block();
            let after_block = builder.new_block();

            // Jump from current to header
            builder.set_terminator(current, Terminator::jump(header));

            // Header: evaluate condition
            let cond_ref = lower_expr(builder, header, &while_stmt.cond);
            builder.set_terminator(header, Terminator::branch(cond_ref, body_block, after_block));

            // Body: `continue` re-evaluates the condition
            let targets = LoopTargets { break_to: after_block, continue_to: header };
            let body_exit = flatten_loop_body(builder, &while_stmt.body, body_block, targets);
            if builder.block_mut(body_exit).term == Terminator::Unreachable {
                builder.set_terminator(body_exit, Terminator::jump(header));
            }

            after_block
        }

        CanonStmt::For(for_stmt) => {
            // Lower for-loop as: init; while(cond) { body; update; }
            let init_exit = match &for_stmt.init {
                Some(init) => flatten_stmt(builder, init, current),
                None => current,
            };

            let header = builder.new_block();
            let body_block = builder.new_block();
            let after_block = builder.new_block();

            builder.set_terminator(init_exit, Terminator::jump(header));

            // Condition
            match &for_stmt.cond {
                Some(cond) => {
                    let cond_ref = lower_expr(builder, header, cond);
                    builder.set_terminator(
                        header,
                        Terminator::branch(cond_ref, body_block, after_block),
                    );
                }
                None => builder.set_terminator(header, Terminator::jump(body_block)),
            }

            // Body, then the update in a separate latch block that
            // `continue` also jumps to
            let latch = builder.new_block();
            let targets = LoopTargets { break_to: after_block, continue_to: latch };
            let body_exit = flatten_loop_body(builder, &for_stmt.body, body_block, targets);
            if builder.block_mut(body_exit).term == Terminator::Unreachable {
                builder.set_terminator(body_exit, Terminator::jump(latch));
            }
            let update_exit = match &for_stmt.update {
                Some(update) => flatten_stmt(builder, update, latch),
                None => latch,
            };
            if builder.block_mut(update_exit).term == Terminator::Unreachable {
                builder.set_terminator(update_exit, Terminator::jump(header));
            }

            after_block
        }

        CanonStmt::Return(ret) => {
            let vals = match &ret.value {
                Some(value) => vec![lower_expr(builder, current, value)],
                None => vec![],
            };
            builder.emit_effect(current, OpKind::Return(vals), ret.span.as_ref());
            builder.set_terminator(current, Terminator::TxnExit { reverted: false });
            // After return, create an unreachable block for subsequent stmts
            builder.new_block()
        }

        CanonStmt::Revert(revert) => {
            lower_exprs(builder, current, &revert.args);
            builder.set_terminator(current, Terminator::TxnExit { reverted: true });
            builder.new_block()
        }

        CanonStmt::Assert(assert_stmt) => {
            let cond_ref = lower_expr(builder, current, &assert_stmt.cond);
            let kind = OpKind::Assert { cond: cond_ref };
            builder.emit_effect(current, kind, assert_stmt.span.as_ref());
            current
        }

        CanonStmt::Break => {
            let target = builder.loops.last().map(|targets| targets.break_to);
            flatten_loop_exit(builder, current, target)
        }

        CanonStmt::Continue => {
            let target = builder.loops.last().map(|targets| targets.continue_to);
            flatten_loop_exit(builder, current, target)
        }

        CanonStmt::Block(stmts) => flatten_stmts(builder, stmts, current),

        CanonStmt::Store(store) => {
            let resource = lower_resource(builder, current, &store.resource);
            let keys = lower_exprs(builder, current, &store.keys);
            let value = store
                .value
                .as_ref()
                .map(|v| lower_expr(builder, current, v));
            let kind = OpKind::Store(StoreOp { resource, keys, value });
            builder.emit_effect(current, kind, store.span.as_ref());
            current
        }

        CanonStmt::Emit(emit) => {
            let args = lower_exprs(builder, current, &emit.args);
            let kind = OpKind::Emit(EmitOp { event: emit.event.clone(), args });
            builder.emit_effect(current, kind, emit.span.as_ref());
            current
        }

        CanonStmt::Dialect(dialect_stmt) => flatten_dialect_stmt(builder, dialect_stmt, current),
    }
}

/// Flatten a loop body with `targets` as the destinations of `break` and
/// `continue`, returning the body's exit block.
fn flatten_loop_body(
    builder: &mut CfgBuilder,
    body: &[CanonStmt],
    entry: BlockId,
    targets: LoopTargets,
) -> BlockId {
    builder.loops.push(targets);
    let exit = flatten_stmts(builder, body, entry);
    builder.loops.pop();
    exit
}

/// Jump from `current` to the loop exit or continuation `target`. Outside a
/// loop (malformed input) the statement has no effect.
fn flatten_loop_exit(
    builder: &mut CfgBuilder,
    current: BlockId,
    target: Option<BlockId>,
) -> BlockId {
    let Some(target) = target else { return current };
    builder.set_terminator(current, Terminator::jump(target));
    // Statements after the jump are unreachable
    builder.new_block()
}

/// Flatten a dialect statement, returning the block to continue from.
fn flatten_dialect_stmt(
    builder: &mut CfgBuilder,
    stmt: &CanonDialectStmt,
    current: BlockId,
) -> BlockId {
    match stmt {
        CanonDialectStmt::Evm(CanonEvmStmt::TryCatch(try_catch)) => {
            flatten_try_catch(builder, try_catch, current)
        }
        CanonDialectStmt::Evm(CanonEvmStmt::Selfdestruct(destruct)) => {
            let recipient = lower_expr(builder, current, &destruct.recipient);
            let kind = OpKind::Dialect(DialectOp::Evm(EvmOp::Selfdestruct(recipient)));
            builder.emit_effect(current, kind, destruct.span.as_ref());
            builder.set_terminator(current, Terminator::TxnExit { reverted: false });
            builder.new_block()
        }
        CanonDialectStmt::Move(CanonMoveStmt::Abort(abort)) => {
            lower_expr(builder, current, &abort.code);
            builder.set_terminator(current, Terminator::TxnExit { reverted: true });
            builder.new_block()
        }
        // Specifications are not part of the executable CFG.
        CanonDialectStmt::Move(CanonMoveStmt::SpecBlock(_)) => current,
    }
}

/// Lower `try guarded returns (...) { body } catch ... { ... }`.
///
/// The guarded call either succeeds (continue with `body`) or fails; on
/// failure, catch clauses are tried in order and an unmatched failure
/// reverts.
fn flatten_try_catch(
    builder: &mut CfgBuilder,
    try_catch: &CanonTryCatchStmt,
    current: BlockId,
) -> BlockId {
    let span = try_catch.span.as_ref();
    let guarded = lower_expr(builder, current, &try_catch.guarded);
    bind_parts(builder, current, &try_catch.returns, "try_return", guarded, span);

    let after_block = builder.new_block();
    let body_block = builder.new_block();
    let body_exit = flatten_stmts(builder, &try_catch.body, body_block);
    if builder.block_mut(body_exit).term == Terminator::Unreachable {
        builder.set_terminator(body_exit, Terminator::jump(after_block));
    }

    let succeeded =
        OpKind::Opaque { description: "try_succeeded".to_string(), operands: vec![guarded] };
    let cond = builder.emit_value(current, succeeded, TMP_NAME, Type::Bool, span);
    let mut dispatch = builder.new_block();
    builder.set_terminator(current, Terminator::branch(cond, body_block, dispatch));

    for clause in &try_catch.catch_clauses {
        let clause_span = clause.span.as_ref();
        let error = clause.error.as_deref().unwrap_or("*");
        let matches =
            OpKind::Opaque { description: format!("catch_matches({error})"), operands: vec![] };
        let cond = builder.emit_value(dispatch, matches, TMP_NAME, Type::Bool, clause_span);

        let catch_block = builder.new_block();
        let reason =
            OpKind::Opaque { description: "catch_reason".to_string(), operands: vec![guarded] };
        let reason = builder.emit_value(catch_block, reason, TMP_NAME, Type::Bytes, clause_span);
        bind_parts(builder, catch_block, &clause.params, "catch_param", reason, clause_span);
        let catch_exit = flatten_stmts(builder, &clause.body, catch_block);
        if builder.block_mut(catch_exit).term == Terminator::Unreachable {
            builder.set_terminator(catch_exit, Terminator::jump(after_block));
        }

        let next = builder.new_block();
        builder.set_terminator(dispatch, Terminator::branch(cond, catch_block, next));
        dispatch = next;
    }
    builder.set_terminator(dispatch, Terminator::TxnExit { reverted: true });

    after_block
}

/// Declare the variables `vars` and bind each to component `i` of `value`
/// (or to `value` itself when there is a single variable).
fn bind_parts(
    builder: &mut CfgBuilder,
    block: BlockId,
    vars: &[(String, Type)],
    what: &str,
    value: OpRef,
    span: Option<&Loc>,
) {
    for (index, (name, ty)) in vars.iter().enumerate() {
        builder.declare_local(name, ty.clone());
        let part = if vars.len() == 1 {
            value
        } else {
            let kind =
                OpKind::Opaque { description: format!("{what} {index}"), operands: vec![value] };
            builder.emit_value(block, kind, name, ty.clone(), span)
        };
        builder.write_variable(block, name, part);
    }
}

/// Assign `value` to a local place: a variable, an element or field of a
/// local aggregate (rebuilding the aggregate), or a tuple of places.
/// Contract state is written by `Store` statements, never through here.
fn assign_local(
    builder: &mut CfgBuilder,
    block: BlockId,
    lhs: &CanonExpr,
    value: OpRef,
    span: Option<&Loc>,
) {
    match lhs {
        CanonExpr::Var(var) => builder.write_variable(block, &var.name, value),
        CanonExpr::IndexAccess(access) => {
            let parts = std::iter::once(&*access.base).chain(access.index.as_deref());
            let mut operands = lower_expr_refs(builder, block, parts.collect());
            operands.push(value);
            let kind = OpKind::Opaque { description: "index_update".to_string(), operands };
            let name = expr_name(&access.base);
            let updated = builder.emit_value(block, kind, &name, access.base.typ(), span);
            assign_local(builder, block, &access.base, updated, span);
        }
        CanonExpr::FieldAccess(access) => {
            let base = lower_expr(builder, block, &access.base);
            let kind = OpKind::Opaque {
                description: format!("field_update .{}", access.field),
                operands: vec![base, value],
            };
            let name = expr_name(&access.base);
            let updated = builder.emit_value(block, kind, &name, access.base.typ(), span);
            assign_local(builder, block, &access.base, updated, span);
        }
        // `(a, b) = value`: CIR represents the tuple as a `__tuple__` call.
        CanonExpr::FunctionCall(call) if is_tuple_callee(&call.callee) => {
            for (index, elem) in call.args.iter().enumerate() {
                let kind = OpKind::Opaque {
                    description: format!("tuple_get {index}"),
                    operands: vec![value],
                };
                let part = builder.emit_value(block, kind, &expr_name(elem), elem.typ(), span);
                assign_local(builder, block, elem, part, span);
            }
        }
        // Other expressions do not denote local places; the assignment has
        // no effect on local state.
        CanonExpr::Lit(_)
        | CanonExpr::BinOp(_)
        | CanonExpr::UnOp(_)
        | CanonExpr::FunctionCall(_)
        | CanonExpr::TypeCast(_)
        | CanonExpr::Load(_)
        | CanonExpr::InternalCall(_)
        | CanonExpr::ExternalCall(_)
        | CanonExpr::Env(_)
        | CanonExpr::Old(_)
        | CanonExpr::Result(_)
        | CanonExpr::Forall { .. }
        | CanonExpr::Exists { .. }
        | CanonExpr::Dialect(_) => {}
    }
}

// ═══════════════════════════════════════════════════════════════════
// Expression lowering
// ═══════════════════════════════════════════════════════════════════

/// Lower an expression into ops, returning a reference to its value.
fn lower_expr(builder: &mut CfgBuilder, block: BlockId, expr: &CanonExpr) -> OpRef {
    lower_expr_named(builder, block, expr, &expr_name(expr), expr.typ())
}

/// Lower expressions in order, returning references to their values.
fn lower_exprs(builder: &mut CfgBuilder, block: BlockId, exprs: &[CanonExpr]) -> Vec<OpRef> {
    lower_expr_refs(builder, block, exprs.iter().collect())
}

fn lower_expr_refs(
    builder: &mut CfgBuilder,
    block: BlockId,
    exprs: Vec<&CanonExpr>,
) -> Vec<OpRef> {
    exprs
        .into_iter()
        .map(|e| lower_expr(builder, block, e))
        .collect()
}

/// Lower an expression whose value is bound to the SSA name `name`.
fn lower_expr_named(
    builder: &mut CfgBuilder,
    block: BlockId,
    expr: &CanonExpr,
    name: &str,
    ty: Type,
) -> OpRef {
    let span = expr.span();
    let kind = match expr {
        CanonExpr::Lit(lit) => OpKind::Const(lit.clone()),
        // A local read is its reaching definition; no op is emitted.
        CanonExpr::Var(var) => return builder.read_variable(block, &var.name),
        CanonExpr::IndexAccess(access) => {
            let parts = std::iter::once(&*access.base).chain(access.index.as_deref());
            let operands = lower_expr_refs(builder, block, parts.collect());
            opaque(expr, operands)
        }
        CanonExpr::FieldAccess(access) => {
            let operand = lower_expr(builder, block, &access.base);
            opaque(expr, vec![operand])
        }
        CanonExpr::BinOp(binop) => {
            let lhs = lower_expr(builder, block, &binop.lhs);
            let rhs = lower_expr(builder, block, &binop.rhs);
            OpKind::BinOp { op: binop.op, lhs, rhs, overflow: binop.overflow }
        }
        CanonExpr::UnOp(unop) => {
            let operand = lower_expr(builder, block, &unop.operand);
            OpKind::UnOp { op: unop.op, operand }
        }
        // Unresolved callee (builtin, library, or type constructor). A
        // callee that is itself an expression is evaluated too, so effects
        // inside it (e.g. a nested call) are not lost.
        CanonExpr::FunctionCall(call) => {
            let callee = match &*call.callee {
                CanonExpr::Var(_) => None,
                callee => Some(lower_expr(builder, block, callee)),
            };
            let args = lower_exprs(builder, block, &call.args);
            let operands = callee.into_iter().chain(args).collect();
            OpKind::Opaque { description: call.callee.to_string(), operands }
        }
        CanonExpr::TypeCast(cast) => {
            let operand = lower_expr(builder, block, &cast.expr);
            opaque(expr, vec![operand])
        }
        CanonExpr::Load(load) => {
            let resource = lower_resource(builder, block, &load.resource);
            let keys = lower_exprs(builder, block, &load.keys);
            OpKind::Load(LoadOp { resource, keys })
        }
        CanonExpr::InternalCall(call) => {
            let args = lower_exprs(builder, block, &call.args);
            let func = FunctionId(format!("{}.{}", builder.contract, call.func));
            OpKind::Call(CallOp { target: CallTarget::Internal(func), args, value: None })
        }
        CanonExpr::ExternalCall(call) => {
            let address = call
                .address
                .as_deref()
                .map(|a| lower_expr(builder, block, a));
            let args = lower_exprs(builder, block, &call.args);
            let value = call.value.as_deref().map(|v| lower_expr(builder, block, v));
            let target = CallTarget::External(ExternalCallee { kind: call.kind, address });
            OpKind::Call(CallOp { target, args, value })
        }
        CanonExpr::Env(env) => OpKind::Env(env.var),
        CanonExpr::Dialect(dialect) => lower_dialect_expr(builder, block, dialect),
        // Specification-only expressions have no executable semantics.
        CanonExpr::Old(_)
        | CanonExpr::Result(_)
        | CanonExpr::Forall { .. }
        | CanonExpr::Exists { .. } => opaque(expr, vec![]),
    };
    builder.emit_value(block, kind, name, ty, span)
}

/// Lower the location family of a `Load` / `Store`.
fn lower_resource(builder: &mut CfgBuilder, block: BlockId, resource: &CanonResource) -> Resource {
    match resource {
        CanonResource::AnchorAccount(account) => {
            Resource::AnchorAccount(lower_expr(builder, block, account))
        }
        CanonResource::MoveGlobal(ty) => Resource::MoveGlobal(ty.clone()),
        CanonResource::StateVar(path) => Resource::StateVar(path.clone()),
    }
}

/// Lower a chain-specific expression to its typed BIR op.
fn lower_dialect_expr(
    builder: &mut CfgBuilder,
    block: BlockId,
    expr: &CanonDialectExpr,
) -> OpKind {
    let op = match &expr.kind {
        CanonDialectKind::Evm(CanonEvmExpr::Builtin(builtin)) => {
            let args = lower_exprs(builder, block, &builtin.args);
            DialectOp::Evm(EvmOp::Builtin(EvmBuiltinOp { builtin: builtin.builtin, args }))
        }
        CanonDialectKind::Evm(CanonEvmExpr::InlineAsm(text)) => {
            DialectOp::Evm(EvmOp::InlineAsm(text.clone()))
        }
        CanonDialectKind::Move(CanonMoveExpr::BorrowGlobalMut(global)) => {
            DialectOp::Move(MoveOp::BorrowGlobalMut(lower_move_global(builder, block, global)))
        }
        CanonDialectKind::Move(CanonMoveExpr::Exists(global)) => {
            DialectOp::Move(MoveOp::Exists(lower_move_global(builder, block, global)))
        }
        CanonDialectKind::Move(CanonMoveExpr::MoveFrom(global)) => {
            DialectOp::Move(MoveOp::MoveFrom(lower_move_global(builder, block, global)))
        }
        CanonDialectKind::Move(CanonMoveExpr::SignerAddress(signer)) => {
            DialectOp::Move(MoveOp::SignerAddress(lower_expr(builder, block, signer)))
        }
        CanonDialectKind::Move(CanonMoveExpr::WriteRef(write)) => {
            let reference = lower_expr(builder, block, &write.reference);
            let value = lower_expr(builder, block, &write.value);
            DialectOp::Move(MoveOp::WriteRef(MoveWriteRefOp { reference, value }))
        }
        CanonDialectKind::Anchor(CanonAnchorExpr::AccountLoadMut(account)) => {
            DialectOp::Anchor(AnchorOp::AccountLoadMut(lower_expr(builder, block, account)))
        }
        CanonDialectKind::Anchor(CanonAnchorExpr::FindProgramAddress(pda)) => {
            let seeds = lower_exprs(builder, block, &pda.seeds);
            let program_id = lower_expr(builder, block, &pda.program_id);
            DialectOp::Anchor(AnchorOp::FindProgramAddress(AnchorPdaOp { program_id, seeds }))
        }
        CanonDialectKind::Anchor(CanonAnchorExpr::SignerKey(account)) => {
            DialectOp::Anchor(AnchorOp::SignerKey(lower_expr(builder, block, account)))
        }
    };
    OpKind::Dialect(op)
}

fn lower_move_global(
    builder: &mut CfgBuilder,
    block: BlockId,
    global: &CanonMoveGlobalExpr,
) -> MoveGlobalOp {
    MoveGlobalOp { addr: lower_expr(builder, block, &global.addr), ty: global.ty.clone() }
}

// ═══════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════

fn attach_span(op: Op, span: Option<&Loc>) -> Op {
    match span {
        Some(span) => op.with_span(span.clone()),
        None => op,
    }
}

fn opaque(expr: &CanonExpr, operands: Vec<OpRef>) -> OpKind {
    OpKind::Opaque { description: expr.to_string(), operands }
}

fn is_tuple_callee(callee: &CanonExpr) -> bool {
    matches!(callee, CanonExpr::Var(var) if var.name == TUPLE_CALLEE)
}

/// Extract a name from an expression (for SSA naming).
fn expr_name(expr: &CanonExpr) -> String {
    match expr {
        CanonExpr::Var(v) => v.name.clone(),
        CanonExpr::IndexAccess(idx) => format!("{}_idx", expr_name(&idx.base)),
        CanonExpr::FieldAccess(field) => format!("{}_{}", expr_name(&field.base), field.field),
        _ => TMP_NAME.to_string(),
    }
}

// ========================================================================
// Tests
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bir::ops::{EnvVar, ExternalKind};
    use crate::cir::lower::test_support::*;
    use crate::sir::evm::{EvmExpr, EvmLowLevelCall, EvmMsgSender};
    use crate::sir::{
        BinOp, DialectExpr, Expr, ForStmt, IfStmt, IndexAccessExpr, ReturnStmt, Stmt, StringLit,
        WhileStmt,
    };

    /// `balances[msg.sender] = false;`
    fn assign_balance_of_sender() -> Stmt {
        let sender = Expr::Dialect(DialectExpr::Evm(EvmExpr::MsgSender(EvmMsgSender {
            loc: Loc::default(),
        })));
        let lhs = Expr::IndexAccess(IndexAccessExpr {
            base: Box::new(var("balances")),
            index: Some(Box::new(sender)),
            ty: Type::I256,
            span: None,
        });
        assign(lhs, lit(false))
    }

    /// `i = false; while (i < n) { i = i + true; }`
    fn counting_loop() -> Vec<Stmt> {
        let body = vec![assign(var("i"), binop(BinOp::Add, var("i"), lit(true)))];
        let cond = binop(BinOp::Lt, var("i"), var("n"));
        vec![
            assign(var("i"), lit(false)),
            Stmt::While(WhileStmt { cond, body, invariant: None, span: None }),
        ]
    }

    /// `x = a; y = a; if (c) { x = b; } return x + y;`
    fn conditional_update() -> Vec<Stmt> {
        let then_body = vec![assign(var("x"), var("b"))];
        let sum = binop(BinOp::Add, var("x"), var("y"));
        vec![
            assign(var("x"), var("a")),
            assign(var("y"), var("a")),
            Stmt::If(IfStmt { cond: var("c"), then_body, else_body: None, span: None }),
            Stmt::Return(ReturnStmt { value: Some(sum), span: None }),
        ]
    }

    fn find_op(blocks: &[BasicBlock], id: OpRef) -> &Op {
        blocks
            .iter()
            .flat_map(|b| &b.ops)
            .find(|op| op.id == id.0)
            .unwrap()
    }

    /// Blocks that take parameters.
    fn param_blocks(blocks: &[BasicBlock]) -> Vec<&BasicBlock> {
        blocks.iter().filter(|b| !b.params.is_empty()).collect()
    }

    /// The ops passed as argument `index` on every transfer into `block`.
    fn incoming<'b>(blocks: &'b [BasicBlock], block: BlockId, index: usize) -> Vec<&'b Op> {
        blocks
            .iter()
            .flat_map(|b| b.term.block_calls())
            .filter(|call| call.block == block)
            .map(|call| find_op(blocks, call.args[index]))
            .collect()
    }

    #[test]
    fn test_call_then_state_write_keeps_operands_and_order() {
        // recipient.call{value: amount}(""); balances[msg.sender] = false;
        let call = Expr::Dialect(DialectExpr::Evm(EvmExpr::LowLevelCall(EvmLowLevelCall {
            target: Box::new(var("recipient")),
            data: Box::new(Expr::Lit(Lit::String(StringLit::new(String::new(), None)))),
            value: Some(Box::new(var("amount"))),
            gas: None,
            loc: Loc::default(),
        })));
        let body = vec![expr_stmt(call), assign_balance_of_sender()];
        let blocks = lower_function(&["balances"], params(&["recipient", "amount"]), body);
        let ops = &blocks[0].ops;

        let call_pos = ops
            .iter()
            .position(|op| matches!(op.kind, OpKind::Call(_)))
            .unwrap();
        let OpKind::Call(call) = &ops[call_pos].kind else {
            unreachable!()
        };
        let CallTarget::External(callee) = &call.target else {
            panic!("expected external call")
        };
        assert_eq!(callee.kind, ExternalKind::Call);
        assert!(callee.address.is_some());
        assert_eq!(call.args.len(), 1);
        assert!(call.value.is_some());

        let store_pos = ops
            .iter()
            .position(|op| matches!(op.kind, OpKind::Store(_)))
            .unwrap();
        let OpKind::Store(store) = &ops[store_pos].kind else {
            unreachable!()
        };
        assert_eq!(store.resource, Resource::StateVar("balances".to_string()));
        assert_eq!(store.keys.len(), 1);
        assert!(matches!(find_op(&blocks, store.keys[0]).kind, OpKind::Env(EnvVar::Caller)));
        assert!(call_pos < store_pos, "state write must stay after the call");
    }

    #[test]
    fn test_local_shadowing_state_var_is_not_storage() {
        // A parameter named like a state variable shadows it.
        let blocks =
            lower_function(&["balances"], params(&["balances"]), vec![assign_balance_of_sender()]);
        let has_storage_op = blocks
            .iter()
            .flat_map(|b| &b.ops)
            .any(|op| op.kind.storage_access().is_some());
        assert!(!has_storage_op);
    }

    #[test]
    fn test_loop_carried_variable_becomes_header_param() {
        let blocks = lower_function(&[], params(&["n"]), counting_loop());

        // Only the loop header merges values of `i`.
        let [header] = param_blocks(&blocks)[..] else {
            panic!("expected one header")
        };
        assert_eq!(header.params.len(), 1);
        let i_param = OpRef(header.params[0].id);

        // The loop condition reads the header parameter.
        let Terminator::Branch { cond, .. } = &header.term else {
            panic!("expected branch")
        };
        let OpKind::BinOp { lhs, .. } = &find_op(&blocks, *cond).kind else {
            panic!()
        };
        assert_eq!(*lhs, i_param);

        // `i` enters as the initial constant and loops back as `i + 1`.
        let args = incoming(&blocks, header.id, 0);
        assert_eq!(args.len(), 2);
        assert!(args.iter().any(|op| matches!(op.kind, OpKind::Const(_))));
        assert!(
            args.iter()
                .any(|op| matches!(op.kind, OpKind::BinOp { lhs, .. } if lhs == i_param))
        );
    }

    #[test]
    fn test_merge_param_only_for_variables_changed_on_a_path() {
        let blocks = lower_function(&[], params(&["a", "b", "c"]), conditional_update());

        // `x` differs between the paths; `y` does not, so it gets no param.
        let [merge] = param_blocks(&blocks)[..] else {
            panic!("expected one merge block")
        };
        assert_eq!(merge.params.len(), 1);
        let x_param = OpRef(merge.params[0].id);
        let indices: HashSet<_> = incoming(&blocks, merge.id, 0)
            .iter()
            .map(|op| match op.kind {
                OpKind::Param { index } => index,
                _ => panic!("expected a function parameter"),
            })
            .collect();
        assert_eq!(indices, HashSet::from([0, 1]));

        // `return x + y` reads the merged `x` and the parameter `a` for `y`.
        let ret = blocks
            .iter()
            .flat_map(|b| &b.ops)
            .find_map(|op| match &op.kind {
                OpKind::Return(vals) => Some(vals[0]),
                _ => None,
            });
        let OpKind::BinOp { lhs, rhs, .. } = &find_op(&blocks, ret.unwrap()).kind else {
            panic!("expected x + y")
        };
        assert_eq!(*lhs, x_param);
        assert!(matches!(find_op(&blocks, *rhs).kind, OpKind::Param { index: 0 }));
    }

    #[test]
    fn test_break_and_continue_jump_to_loop_exit_and_latch() {
        // for (i = false; i < n; i = i + true) { if (c) continue; if (d) break;
        // }
        let jump_if = |cond: &str, stmt: Stmt| {
            Stmt::If(IfStmt {
                cond: var(cond),
                then_body: vec![stmt],
                else_body: None,
                span: None,
            })
        };
        let body = vec![Stmt::For(ForStmt {
            init: Some(Box::new(assign(var("i"), lit(false)))),
            cond: Some(binop(BinOp::Lt, var("i"), var("n"))),
            update: Some(Box::new(assign(var("i"), binop(BinOp::Add, var("i"), lit(true))))),
            body: vec![jump_if("c", Stmt::Continue), jump_if("d", Stmt::Break)],
            invariant: None,
            span: None,
        })];
        let blocks = lower_function(&[], params(&["n", "c", "d"]), body);
        let transfers_into = |target: BlockId| {
            blocks
                .iter()
                .flat_map(|b| b.term.successors())
                .filter(|s| *s == target)
                .count()
        };

        // The loop exit is reached when the condition fails and on `break`.
        let header = blocks
            .iter()
            .find(|b| {
                matches!(&b.term, Terminator::Branch { cond, .. }
                if matches!(find_op(&blocks, *cond).kind, OpKind::BinOp { op: BinOp::Lt, .. }))
            })
            .unwrap();
        let Terminator::Branch { else_dest, .. } = &header.term else {
            unreachable!()
        };
        assert_eq!(transfers_into(else_dest.block), 2);

        // The latch runs the update; it is reached on `continue` and at the
        // end of the body, and loops back to the header.
        let latch = blocks
            .iter()
            .find(|b| {
                b.ops
                    .iter()
                    .any(|op| matches!(op.kind, OpKind::BinOp { op: BinOp::Add, .. }))
            })
            .unwrap();
        assert_eq!(transfers_into(latch.id), 2);
        assert_eq!(latch.term.successors(), vec![header.id]);
    }

    #[test]
    fn test_ssa_output_passes_bir_verifier() {
        let functions = vec![
            TestFunction {
                name: "f",
                params: params(&["n"]),
                body: counting_loop(),
                public: true,
            },
            TestFunction {
                name: "g",
                params: params(&["a", "b", "c"]),
                body: conditional_update(),
                public: false,
            },
        ];
        let module = lower_contract(&[], functions);
        assert!(crate::bir::verifier::verify(&module, false).is_ok());
    }
}
