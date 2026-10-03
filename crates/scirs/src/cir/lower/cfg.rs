//! Step 1: CFG Construction and Typed Op Lowering
//!
//! Converts structured SIR statements into basic blocks with explicit
//! control flow edges (terminators). Expressions are lowered directly into
//! typed BIR ops: contract state accesses become `Load`/`Store`, calls
//! become `Call` with their arguments, environment reads become `Env`, and
//! dialect constructs become shared feature ops or typed `Dialect` ops.

use crate::bir::cfg::{BasicBlock, BlockId, BlockParam, FunctionId, Terminator};
use crate::bir::ops::{
    AnchorOp, AnchorPdaOp, CallOp, CallTarget, DialectOp, EmitOp, EnvVar, EvmBuiltin,
    EvmBuiltinOp, EvmOp, ExternalCallee, ExternalKind, LoadOp, MoveGlobalOp, MoveOp,
    MoveWriteRefOp, Op, OpId, OpKind, OpRef, Resource, SsaName, StoreOp,
};
use crate::sir::anchor::{AnchorExpr, AnchorStmt};
use crate::sir::evm::{EvmExpr, EvmStmt, EvmTryCatch};
use crate::sir::move_lang::{MoveExpr, MoveStmt};
use crate::sir::{BoolLit, CallExpr, DialectExpr, DialectStmt, Expr, Lit, Loc, Param, Stmt, Type};
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

// ═══════════════════════════════════════════════════════════════════
// Data Structures
// ═══════════════════════════════════════════════════════════════════

/// Contract-level names needed to classify accesses and calls.
pub struct ContractScope {
    /// Contract name, used to qualify internal function ids.
    pub contract: String,
    /// Names of the contract's functions (internal call targets).
    pub functions: HashSet<String>,
    /// Names of the contract's state variables.
    pub storage_vars: HashSet<String>,
}

/// Where `break` and `continue` jump inside a loop.
#[derive(Clone, Copy)]
struct LoopTargets {
    break_to: BlockId,
    continue_to: BlockId,
}

/// State for the CFG builder.
struct CfgBuilder<'s> {
    blocks: Vec<BasicBlock>,
    /// Current SSA definition of each local variable, per block.
    current_defs: HashMap<BlockId, HashMap<String, OpRef>>,
    /// Parameters and local variables; they shadow state variables.
    locals: HashSet<String>,
    /// Enclosing loops, innermost last.
    loops: Vec<LoopTargets>,
    next_block_id: usize,
    next_op_id: usize,
    /// Block parameters awaiting incoming arguments: (block, variable, index).
    pending_params: Vec<(BlockId, String, usize)>,
    /// Distinct predecessors of each block; `None` until the CFG is complete
    /// (all blocks are unsealed while statements are being flattened).
    preds: Option<HashMap<BlockId, Vec<BlockId>>>,
    scope: &'s ContractScope,
    /// `Symbol` ops (in the entry block) for names with no local definition.
    symbols: HashMap<String, OpRef>,
    /// Declared type of each local variable, used for block parameters.
    var_types: HashMap<String, Type>,
}

impl<'s> CfgBuilder<'s> {
    fn new(scope: &'s ContractScope) -> Self {
        CfgBuilder {
            blocks: Vec::new(),
            current_defs: HashMap::new(),
            locals: HashSet::new(),
            loops: Vec::new(),
            next_block_id: 0,
            next_op_id: 0,
            pending_params: Vec::new(),
            preds: None,
            scope,
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

    fn is_state_var(&self, name: &str) -> bool {
        self.scope.storage_vars.contains(name) && !self.locals.contains(name)
    }

    /// If `expr` denotes contract state, return its path (struct fields
    /// joined by `.`) and its index expressions in order.
    fn storage_path<'e>(&self, expr: &'e Expr) -> Option<(String, Vec<&'e Expr>)> {
        match expr {
            Expr::Var(var) if self.is_state_var(&var.name) => Some((var.name.clone(), vec![])),
            Expr::IndexAccess(access) => {
                let (path, mut keys) = self.storage_path(&access.base)?;
                keys.extend(access.index.as_deref());
                Some((path, keys))
            }
            Expr::FieldAccess(access) => {
                let (path, keys) = self.storage_path(&access.base)?;
                Some((format!("{path}.{}", access.field), keys))
            }
            _ => None,
        }
    }

    /// Returns `true` if `expr` evaluates to a value (as opposed to naming a
    /// contract, library, or type), so a member call on it is external.
    fn is_value_receiver(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Var(var) => self.locals.contains(&var.name) || self.is_state_var(&var.name),
            _ => true,
        }
    }

    fn declare_local(&mut self, name: &str, ty: Type) {
        self.locals.insert(name.to_string());
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
        self.current_defs.entry(block).or_default().insert(name.to_string(), value);
    }

    /// The SSA value of local variable `name` at the current end of `block`.
    fn read_variable(&mut self, block: BlockId, name: &str) -> OpRef {
        if let Some(value) = self.current_defs.get(&block).and_then(|defs| defs.get(name)) {
            return *value;
        }
        let preds = self.preds.as_ref().map(|preds| preds.get(&block).cloned().unwrap_or_default());
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
        self.preds.as_ref().and_then(|preds| preds.get(&block).cloned()).unwrap_or_default()
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

/// Build a CFG from a list of SIR statements (function body).
pub fn build_cfg(stmts: &[Stmt], params: &[Param], scope: &ContractScope) -> Vec<BasicBlock> {
    let mut builder = CfgBuilder::new(scope);
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
fn flatten_stmts(builder: &mut CfgBuilder, stmts: &[Stmt], mut current: BlockId) -> BlockId {
    for stmt in stmts {
        current = flatten_stmt(builder, stmt, current);
    }
    current
}

/// Flatten a single statement, returning the block to continue from.
fn flatten_stmt(builder: &mut CfgBuilder, stmt: &Stmt, current: BlockId) -> BlockId {
    match stmt {
        Stmt::LocalVar(local_var) => {
            let span = local_var.span.as_ref();
            let decls: Vec<_> = local_var.vars.iter().collect();
            for decl in decls.iter().copied().flatten() {
                builder.declare_local(&decl.name, decl.ty.clone());
            }
            match (decls.as_slice(), &local_var.init) {
                // `T x = init;`
                ([Some(decl)], Some(init)) => {
                    let value = lower_expr_named(builder, current, init, &decl.name, decl.ty.clone());
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
                        let part = builder.emit_value(current, part, &decl.name, decl.ty.clone(), span);
                        builder.write_variable(current, &decl.name, part);
                    }
                }
                // `T x;` declares default-initialized variables.
                (_, None) => {
                    for decl in decls.iter().copied().flatten() {
                        let kind = OpKind::Const(Lit::Bool(BoolLit::new(false, None)));
                        let value = builder.emit_value(current, kind, &decl.name, decl.ty.clone(), span);
                        builder.write_variable(current, &decl.name, value);
                    }
                }
            }
            current
        }

        Stmt::Assign(assign) => {
            if let Some((path, key_exprs)) = builder.storage_path(&assign.lhs) {
                let value = lower_expr(builder, current, &assign.rhs);
                let keys = lower_exprs(builder, current, key_exprs);
                let kind = OpKind::Store(StoreOp {
                    resource: Resource::StateVar(path),
                    keys,
                    value: Some(value),
                });
                builder.emit_effect(current, kind, assign.span.as_ref());
            } else {
                let name = expr_name(&assign.lhs);
                let value =
                    lower_expr_named(builder, current, &assign.rhs, &name, assign.rhs.typ());
                assign_local(builder, current, &assign.lhs, value, assign.span.as_ref());
            }
            current
        }

        Stmt::AugAssign(aug) => {
            // `lhs op= rhs` reads lhs (a Load if it is state), computes, then
            // writes back (a Store if it is state).
            let lhs_ref = lower_expr(builder, current, &aug.lhs);
            let rhs_ref = lower_expr(builder, current, &aug.rhs);
            let kind = OpKind::BinOp {
                op: aug.op,
                lhs: lhs_ref,
                rhs: rhs_ref,
                overflow: crate::sir::OverflowSemantics::Checked,
            };
            let name = expr_name(&aug.lhs);
            let span = aug.span.as_ref();
            let value = builder.emit_value(current, kind, &name, aug.lhs.typ(), span);
            if let Some((path, key_exprs)) = builder.storage_path(&aug.lhs) {
                let keys = lower_exprs(builder, current, key_exprs);
                let kind = OpKind::Store(StoreOp {
                    resource: Resource::StateVar(path),
                    keys,
                    value: Some(value),
                });
                builder.emit_effect(current, kind, span);
            } else {
                assign_local(builder, current, &aug.lhs, value, span);
            }
            current
        }

        Stmt::Expr(expr_stmt) => {
            let expr_ref = lower_expr(builder, current, &expr_stmt.expr);
            let kind = OpKind::ExprStmt { expr: expr_ref };
            builder.emit_effect(current, kind, expr_stmt.span.as_ref());
            current
        }

        Stmt::If(if_stmt) => {
            let cond_ref = lower_expr(builder, current, &if_stmt.cond);

            let then_block = builder.new_block();
            let merge_block = builder.new_block();

            let then_exit = flatten_stmts(builder, &if_stmt.then_body, then_block);
            if builder.block_mut(then_exit).term == Terminator::Unreachable {
                builder.set_terminator(then_exit, Terminator::jump(merge_block));
            }

            if let Some(else_body) = &if_stmt.else_body {
                let else_block = builder.new_block();
                let else_exit = flatten_stmts(builder, else_body, else_block);
                if builder.block_mut(else_exit).term == Terminator::Unreachable {
                    builder.set_terminator(else_exit, Terminator::jump(merge_block));
                }
                builder.set_terminator(
                    current,
                    Terminator::branch(cond_ref, then_block, else_block),
                );
            } else {
                builder.set_terminator(
                    current,
                    Terminator::branch(cond_ref, then_block, merge_block),
                );
            }

            merge_block
        }

        Stmt::While(while_stmt) => {
            let header = builder.new_block();
            let body_block = builder.new_block();
            let after_block = builder.new_block();

            // Jump from current to header
            builder.set_terminator(current, Terminator::jump(header));

            // Header: evaluate condition
            let cond_ref = lower_expr(builder, header, &while_stmt.cond);
            builder.set_terminator(
                header,
                Terminator::branch(cond_ref, body_block, after_block),
            );

            // Body: `continue` re-evaluates the condition
            let targets = LoopTargets { break_to: after_block, continue_to: header };
            let body_exit = flatten_loop_body(builder, &while_stmt.body, body_block, targets);
            if builder.block_mut(body_exit).term == Terminator::Unreachable {
                builder.set_terminator(body_exit, Terminator::jump(header));
            }

            after_block
        }

        Stmt::For(for_stmt) => {
            // Lower for-loop as: init; while(cond) { body; update; }
            let init_exit = if let Some(init) = &for_stmt.init {
                flatten_stmt(builder, init, current)
            } else {
                current
            };

            let header = builder.new_block();
            let body_block = builder.new_block();
            let after_block = builder.new_block();

            builder.set_terminator(init_exit, Terminator::jump(header));

            // Condition
            if let Some(cond) = &for_stmt.cond {
                let cond_ref = lower_expr(builder, header, cond);
                builder.set_terminator(header, Terminator::branch(cond_ref, body_block, after_block));
            } else {
                builder.set_terminator(header, Terminator::jump(body_block));
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

        Stmt::Return(ret) => {
            let vals = match &ret.value {
                Some(value) => vec![lower_expr(builder, current, value)],
                None => vec![],
            };
            builder.emit_effect(current, OpKind::Return(vals), ret.span.as_ref());
            builder.set_terminator(current, Terminator::TxnExit { reverted: false });
            // After return, create an unreachable block for subsequent stmts
            builder.new_block()
        }

        Stmt::Revert(revert) => {
            let args = revert.args.iter().collect();
            lower_exprs(builder, current, args);
            builder.set_terminator(current, Terminator::TxnExit { reverted: true });
            builder.new_block()
        }

        Stmt::Assert(assert_stmt) => {
            let cond_ref = lower_expr(builder, current, &assert_stmt.cond);
            let kind = OpKind::Assert { cond: cond_ref };
            builder.emit_effect(current, kind, assert_stmt.span.as_ref());
            current
        }

        Stmt::Break => {
            let target = builder.loops.last().map(|targets| targets.break_to);
            flatten_loop_exit(builder, current, target)
        }

        Stmt::Continue => {
            let target = builder.loops.last().map(|targets| targets.continue_to);
            flatten_loop_exit(builder, current, target)
        }

        Stmt::Block(stmts) => flatten_stmts(builder, stmts, current),

        Stmt::Dialect(dialect_stmt) => flatten_dialect_stmt(builder, dialect_stmt, current),
    }
}

/// Flatten a loop body with `targets` as the destinations of `break` and
/// `continue`, returning the body's exit block.
fn flatten_loop_body(
    builder: &mut CfgBuilder,
    body: &[Stmt],
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
fn flatten_loop_exit(builder: &mut CfgBuilder, current: BlockId, target: Option<BlockId>) -> BlockId {
    let Some(target) = target else { return current };
    builder.set_terminator(current, Terminator::jump(target));
    // Statements after the jump are unreachable
    builder.new_block()
}

/// Flatten a dialect statement, returning the block to continue from.
fn flatten_dialect_stmt(builder: &mut CfgBuilder, stmt: &DialectStmt, current: BlockId) -> BlockId {
    match stmt {
        DialectStmt::Evm(EvmStmt::EmitEvent(emit)) => {
            let args = lower_exprs(builder, current, emit.args.iter().collect());
            let kind = OpKind::Emit(EmitOp { event: emit.event.clone(), args });
            builder.emit_effect(current, kind, Some(&emit.loc));
            current
        }
        DialectStmt::Evm(EvmStmt::TryCatch(try_catch)) => {
            flatten_try_catch(builder, try_catch, current)
        }
        // Modifier placeholders are expanded away by CIR.
        DialectStmt::Evm(EvmStmt::Placeholder(_)) => current,
        DialectStmt::Evm(EvmStmt::Selfdestruct(destruct)) => {
            let recipient = lower_expr(builder, current, &destruct.recipient);
            let kind = OpKind::Dialect(DialectOp::Evm(EvmOp::Selfdestruct(recipient)));
            builder.emit_effect(current, kind, Some(&destruct.loc));
            builder.set_terminator(current, Terminator::TxnExit { reverted: false });
            builder.new_block()
        }
        DialectStmt::Move(MoveStmt::Abort(abort)) => {
            lower_expr(builder, current, &abort.expr);
            builder.set_terminator(current, Terminator::TxnExit { reverted: true });
            builder.new_block()
        }
        // Specifications are not part of the executable CFG.
        DialectStmt::Move(MoveStmt::SpecBlock(_)) => current,
        DialectStmt::Anchor(AnchorStmt::EmitEvent(emit)) => {
            let fields = emit.fields.iter().map(|(_, expr)| expr).collect();
            let args = lower_exprs(builder, current, fields);
            let kind = OpKind::Emit(EmitOp { event: emit.event.clone(), args });
            builder.emit_effect(current, kind, Some(&emit.loc));
            current
        }
    }
}

/// Lower `try guarded returns (...) { body } catch ... { ... }`.
///
/// The guarded call either succeeds (continue with `body`) or fails; on
/// failure, catch clauses are tried in order and an unmatched failure
/// reverts.
fn flatten_try_catch(builder: &mut CfgBuilder, try_catch: &EvmTryCatch, current: BlockId) -> BlockId {
    let span = Some(&try_catch.loc);
    let guarded = lower_expr(builder, current, &try_catch.guarded_expr);
    bind_parts(builder, current, &try_catch.returns, "try_return", guarded, span);

    let after_block = builder.new_block();
    let body_block = builder.new_block();
    let body_exit = flatten_stmts(builder, &try_catch.body, body_block);
    if builder.block_mut(body_exit).term == Terminator::Unreachable {
        builder.set_terminator(body_exit, Terminator::jump(after_block));
    }

    let succeeded = OpKind::Opaque {
        description: "try_succeeded".to_string(),
        operands: vec![guarded],
    };
    let cond = builder.emit_value(current, succeeded, TMP_NAME, Type::Bool, span);
    let mut dispatch = builder.new_block();
    builder.set_terminator(
        current,
        Terminator::branch(cond, body_block, dispatch),
    );

    for clause in &try_catch.catch_clauses {
        let error = clause.error.as_deref().unwrap_or("*");
        let matches = OpKind::Opaque { description: format!("catch_matches({error})"), operands: vec![] };
        let cond = builder.emit_value(dispatch, matches, TMP_NAME, Type::Bool, Some(&clause.loc));

        let catch_block = builder.new_block();
        let reason = OpKind::Opaque { description: "catch_reason".to_string(), operands: vec![guarded] };
        let reason = builder.emit_value(catch_block, reason, TMP_NAME, Type::Bytes, Some(&clause.loc));
        bind_parts(builder, catch_block, &clause.params, "catch_param", reason, Some(&clause.loc));
        let catch_exit = flatten_stmts(builder, &clause.body, catch_block);
        if builder.block_mut(catch_exit).term == Terminator::Unreachable {
            builder.set_terminator(catch_exit, Terminator::jump(after_block));
        }

        let next = builder.new_block();
        builder.set_terminator(
            dispatch,
            Terminator::branch(cond, catch_block, next),
        );
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

/// Assign `value` to a non-storage place: a local variable, an element or
/// field of a local aggregate (rebuilding the aggregate), or a tuple of
/// places.
fn assign_local(
    builder: &mut CfgBuilder,
    block: BlockId,
    lhs: &Expr,
    value: OpRef,
    span: Option<&Loc>,
) {
    match lhs {
        Expr::Var(var) => builder.write_variable(block, &var.name, value),
        Expr::IndexAccess(access) => {
            let parts = std::iter::once(&*access.base).chain(access.index.as_deref());
            let mut operands = lower_exprs(builder, block, parts.collect());
            operands.push(value);
            let kind = OpKind::Opaque { description: "index_update".to_string(), operands };
            let name = expr_name(&access.base);
            let updated = builder.emit_value(block, kind, &name, access.base.typ(), span);
            assign_local(builder, block, &access.base, updated, span);
        }
        Expr::FieldAccess(access) => {
            let base = lower_expr(builder, block, &access.base);
            let kind = OpKind::Opaque {
                description: format!("field_update .{}", access.field),
                operands: vec![base, value],
            };
            let name = expr_name(&access.base);
            let updated = builder.emit_value(block, kind, &name, access.base.typ(), span);
            assign_local(builder, block, &access.base, updated, span);
        }
        Expr::Tuple(tuple) => {
            for (index, elem) in tuple.elems.iter().enumerate() {
                let Some(elem) = elem else { continue };
                let kind = OpKind::Opaque {
                    description: format!("tuple_get {index}"),
                    operands: vec![value],
                };
                let part = builder.emit_value(block, kind, &expr_name(elem), elem.typ(), span);
                assign_local(builder, block, elem, part, span);
            }
        }
        // Other expressions do not denote places; the assignment has no
        // effect on local state.
        Expr::Lit(_)
        | Expr::BinOp(_)
        | Expr::UnOp(_)
        | Expr::FunctionCall(_)
        | Expr::TypeCast(_)
        | Expr::Ternary(_)
        | Expr::Old(_)
        | Expr::Result(_)
        | Expr::Forall { .. }
        | Expr::Exists { .. }
        | Expr::Dialect(_) => {}
    }
}

// ═══════════════════════════════════════════════════════════════════
// Expression lowering
// ═══════════════════════════════════════════════════════════════════

/// Lower an expression into ops, returning a reference to its value.
fn lower_expr(builder: &mut CfgBuilder, block: BlockId, expr: &Expr) -> OpRef {
    lower_expr_named(builder, block, expr, &expr_name(expr), expr.typ())
}

/// Lower expressions in order, returning references to their values.
fn lower_exprs(builder: &mut CfgBuilder, block: BlockId, exprs: Vec<&Expr>) -> Vec<OpRef> {
    exprs.into_iter().map(|e| lower_expr(builder, block, e)).collect()
}

/// Lower an expression whose value is bound to the SSA name `name`.
fn lower_expr_named(
    builder: &mut CfgBuilder,
    block: BlockId,
    expr: &Expr,
    name: &str,
    ty: Type,
) -> OpRef {
    let span = expr.span();
    let kind = match expr {
        Expr::Lit(lit) => OpKind::Const(lit.clone()),
        Expr::Var(var) if builder.is_state_var(&var.name) => OpKind::Load(LoadOp {
            resource: Resource::StateVar(var.name.clone()),
            keys: vec![],
        }),
        // A local read is its reaching definition; no op is emitted.
        Expr::Var(var) => return builder.read_variable(block, &var.name),
        Expr::IndexAccess(access) => match builder.storage_path(expr) {
            Some((path, key_exprs)) => load_state_var(builder, block, path, key_exprs),
            None => {
                let parts = std::iter::once(&*access.base).chain(access.index.as_deref());
                let operands = lower_exprs(builder, block, parts.collect());
                opaque(expr, operands)
            }
        },
        Expr::FieldAccess(access) => match builder.storage_path(expr) {
            Some((path, key_exprs)) => load_state_var(builder, block, path, key_exprs),
            None => {
                let operand = lower_expr(builder, block, &access.base);
                opaque(expr, vec![operand])
            }
        },
        Expr::BinOp(binop) => {
            let lhs = lower_expr(builder, block, &binop.lhs);
            let rhs = lower_expr(builder, block, &binop.rhs);
            OpKind::BinOp { op: binop.op, lhs, rhs, overflow: binop.overflow }
        }
        Expr::UnOp(unop) => {
            let operand = lower_expr(builder, block, &unop.operand);
            OpKind::UnOp { op: unop.op, operand }
        }
        Expr::FunctionCall(call) => lower_call(builder, block, call),
        Expr::TypeCast(cast) => {
            let operand = lower_expr(builder, block, &cast.expr);
            opaque(expr, vec![operand])
        }
        Expr::Ternary(ternary) => {
            let parts = vec![&*ternary.cond, &*ternary.then_expr, &*ternary.else_expr];
            let operands = lower_exprs(builder, block, parts);
            opaque(expr, operands)
        }
        Expr::Tuple(tuple) => {
            let elems = tuple.elems.iter().flatten().collect();
            let operands = lower_exprs(builder, block, elems);
            opaque(expr, operands)
        }
        Expr::Dialect(dialect) => match lower_dialect_expr(builder, block, dialect) {
            Lowered::Kind(kind) => kind,
            Lowered::Value(value) => return value,
        },
        // Specification-only expressions have no executable semantics.
        Expr::Old(_) | Expr::Result(_) | Expr::Forall { .. } | Expr::Exists { .. } => {
            opaque(expr, vec![])
        }
    };
    builder.emit_value(block, kind, name, ty, span)
}

/// Lower a read of the contract state location `path[keys...]`.
fn load_state_var(
    builder: &mut CfgBuilder,
    block: BlockId,
    path: String,
    key_exprs: Vec<&Expr>,
) -> OpKind {
    let keys = lower_exprs(builder, block, key_exprs);
    OpKind::Load(LoadOp { resource: Resource::StateVar(path), keys })
}

/// Lower a function call into a `Call` op when its target is known.
fn lower_call(builder: &mut CfgBuilder, block: BlockId, call: &CallExpr) -> OpKind {
    match &*call.callee {
        Expr::Var(var) if builder.scope.functions.contains(&var.name) => {
            let args = lower_exprs(builder, block, call.args.exprs());
            let func = FunctionId(format!("{}.{}", builder.scope.contract, var.name));
            OpKind::Call(CallOp { target: CallTarget::Internal(func), args, value: None })
        }
        Expr::FieldAccess(access) if builder.is_value_receiver(&access.base) => {
            let address = lower_expr(builder, block, &access.base);
            let mut args = lower_exprs(builder, block, call.args.exprs());
            let (kind, value) = match access.field.as_str() {
                "call" => (ExternalKind::Call, None),
                "delegatecall" => (ExternalKind::DelegateCall, None),
                "staticcall" => (ExternalKind::StaticCall, None),
                "transfer" if args.len() == 1 => (ExternalKind::Transfer, args.pop()),
                "send" if args.len() == 1 => (ExternalKind::Send, args.pop()),
                _ => (ExternalKind::HighLevel, None),
            };
            external_call(kind, Some(address), args, value)
        }
        // Unresolved callee (builtin, library, or type constructor).
        callee => {
            let operands = lower_exprs(builder, block, call.args.exprs());
            OpKind::Opaque { description: callee.to_string(), operands }
        }
    }
}

/// Result of lowering a dialect expression.
enum Lowered {
    /// A new op to emit.
    Kind(OpKind),
    /// An existing value (the expression is a transparent wrapper).
    Value(OpRef),
}

/// Lower a dialect expression.
fn lower_dialect_expr(builder: &mut CfgBuilder, block: BlockId, expr: &DialectExpr) -> Lowered {
    match expr {
        DialectExpr::Evm(evm) => Lowered::Kind(lower_evm_expr(builder, block, evm)),
        DialectExpr::Move(mv) => Lowered::Kind(lower_move_expr(builder, block, mv)),
        DialectExpr::Anchor(anchor) => lower_anchor_expr(builder, block, anchor),
    }
}

/// Lower an EVM dialect expression.
fn lower_evm_expr(builder: &mut CfgBuilder, block: BlockId, expr: &EvmExpr) -> OpKind {
    match expr {
        // ── Environment reads ──────────────────────────────
        EvmExpr::BlockBasefee(_) => OpKind::Env(EnvVar::BlockBasefee),
        EvmExpr::BlockChainid(_) => OpKind::Env(EnvVar::BlockChainid),
        EvmExpr::BlockCoinbase(_) => OpKind::Env(EnvVar::BlockCoinbase),
        EvmExpr::BlockDifficulty(_) => OpKind::Env(EnvVar::BlockDifficulty),
        EvmExpr::BlockGaslimit(_) => OpKind::Env(EnvVar::BlockGaslimit),
        EvmExpr::BlockNumber(_) => OpKind::Env(EnvVar::BlockNumber),
        EvmExpr::Gasleft(_) => OpKind::Env(EnvVar::GasLeft),
        EvmExpr::MsgData(_) => OpKind::Env(EnvVar::CallData),
        EvmExpr::MsgSender(_) => OpKind::Env(EnvVar::Caller),
        EvmExpr::MsgSig(_) => OpKind::Env(EnvVar::Selector),
        EvmExpr::MsgValue(_) => OpKind::Env(EnvVar::CallValue),
        EvmExpr::SelfBalance(_) => OpKind::Env(EnvVar::SelfBalance),
        EvmExpr::This(_) => OpKind::Env(EnvVar::SelfAddress),
        EvmExpr::Timestamp(_) => OpKind::Env(EnvVar::Timestamp),
        EvmExpr::TxOrigin(_) => OpKind::Env(EnvVar::Origin),

        // ── External calls ─────────────────────────────────
        EvmExpr::Delegatecall(call) => {
            let address = lower_expr(builder, block, &call.target);
            let args = vec![lower_expr(builder, block, &call.data)];
            external_call(ExternalKind::DelegateCall, Some(address), args, None)
        }
        EvmExpr::LowLevelCall(call) => {
            let address = lower_expr(builder, block, &call.target);
            let args = vec![lower_expr(builder, block, &call.data)];
            let value = call.value.as_deref().map(|v| lower_expr(builder, block, v));
            external_call(ExternalKind::Call, Some(address), args, value)
        }
        EvmExpr::RawCall(call) => {
            let address = lower_expr(builder, block, &call.target);
            let args = vec![lower_expr(builder, block, &call.data)];
            let value = call.value.as_deref().map(|v| lower_expr(builder, block, v));
            external_call(ExternalKind::Call, Some(address), args, value)
        }
        EvmExpr::Send(send) => {
            let address = lower_expr(builder, block, &send.target);
            let value = lower_expr(builder, block, &send.value);
            external_call(ExternalKind::Send, Some(address), vec![], Some(value))
        }
        EvmExpr::Transfer(transfer) => {
            let address = lower_expr(builder, block, &transfer.target);
            let value = lower_expr(builder, block, &transfer.amount);
            external_call(ExternalKind::Transfer, Some(address), vec![], Some(value))
        }

        // ── Builtins ───────────────────────────────────────
        EvmExpr::AbiDecode(e) => evm_builtin(builder, block, EvmBuiltin::AbiDecode, vec![&e.data]),
        EvmExpr::AbiEncode(e) => {
            evm_builtin(builder, block, EvmBuiltin::AbiEncode, e.args.iter().collect())
        }
        EvmExpr::AbiEncodeCall(e) => {
            let args = std::iter::once(&*e.func).chain(&e.args).collect();
            evm_builtin(builder, block, EvmBuiltin::AbiEncodeCall, args)
        }
        EvmExpr::AbiEncodePacked(e) => {
            evm_builtin(builder, block, EvmBuiltin::AbiEncodePacked, e.args.iter().collect())
        }
        EvmExpr::AbiEncodeWithSelector(e) => {
            let args = std::iter::once(&*e.selector).chain(&e.args).collect();
            evm_builtin(builder, block, EvmBuiltin::AbiEncodeWithSelector, args)
        }
        EvmExpr::AbiEncodeWithSignature(e) => {
            let args = std::iter::once(&*e.signature).chain(&e.args).collect();
            evm_builtin(builder, block, EvmBuiltin::AbiEncodeWithSignature, args)
        }
        EvmExpr::Addmod(e) => evm_builtin(builder, block, EvmBuiltin::Addmod, vec![&e.x, &e.y, &e.k]),
        EvmExpr::Blockhash(e) => evm_builtin(builder, block, EvmBuiltin::Blockhash, vec![&e.expr]),
        EvmExpr::Concat(e) => evm_builtin(builder, block, EvmBuiltin::Concat, e.exprs.iter().collect()),
        EvmExpr::Convert(e) => evm_builtin(builder, block, EvmBuiltin::Convert, vec![&e.expr]),
        EvmExpr::Ecrecover(e) => {
            evm_builtin(builder, block, EvmBuiltin::Ecrecover, vec![&e.hash, &e.v, &e.r, &e.s])
        }
        EvmExpr::Empty(_) => evm_builtin(builder, block, EvmBuiltin::Empty, vec![]),
        EvmExpr::Keccak256(e) => evm_builtin(builder, block, EvmBuiltin::Keccak256, vec![&e.expr]),
        EvmExpr::Len(e) => evm_builtin(builder, block, EvmBuiltin::Len, vec![&e.expr]),
        EvmExpr::Mulmod(e) => evm_builtin(builder, block, EvmBuiltin::Mulmod, vec![&e.x, &e.y, &e.k]),
        EvmExpr::Ripemd160(e) => evm_builtin(builder, block, EvmBuiltin::Ripemd160, vec![&e.expr]),
        EvmExpr::Sha256(e) => evm_builtin(builder, block, EvmBuiltin::Sha256, vec![&e.expr]),
        EvmExpr::Slice(e) => {
            evm_builtin(builder, block, EvmBuiltin::Slice, vec![&e.expr, &e.start, &e.length])
        }

        // ── Other ──────────────────────────────────────────
        EvmExpr::InlineAsm(asm) => OpKind::Dialect(DialectOp::Evm(EvmOp::InlineAsm(asm.asm_text.clone()))),
        // `super` only appears as a callee; CIR resolves inheritance.
        EvmExpr::Super(_) => OpKind::Opaque { description: "evm.super".to_string(), operands: vec![] },
    }
}

/// Lower a Move dialect expression.
fn lower_move_expr(builder: &mut CfgBuilder, block: BlockId, expr: &MoveExpr) -> OpKind {
    match expr {
        MoveExpr::BorrowGlobal(e) => {
            let addr = lower_expr(builder, block, &e.addr);
            OpKind::Load(LoadOp { resource: Resource::MoveGlobal(e.ty.clone()), keys: vec![addr] })
        }
        MoveExpr::BorrowGlobalMut(e) => {
            let addr = lower_expr(builder, block, &e.addr);
            move_op(MoveOp::BorrowGlobalMut(MoveGlobalOp { addr, ty: e.ty.clone() }))
        }
        MoveExpr::Exists(e) => {
            let addr = lower_expr(builder, block, &e.addr);
            move_op(MoveOp::Exists(MoveGlobalOp { addr, ty: e.ty.clone() }))
        }
        MoveExpr::GhostVar(e) => OpKind::Symbol { name: e.name.clone() },
        MoveExpr::MoveFrom(e) => {
            let addr = lower_expr(builder, block, &e.addr);
            move_op(MoveOp::MoveFrom(MoveGlobalOp { addr, ty: e.ty.clone() }))
        }
        MoveExpr::MoveTo(e) => {
            let value = lower_expr(builder, block, &e.resource);
            let signer = lower_expr(builder, block, &e.signer);
            OpKind::Store(StoreOp {
                resource: Resource::MoveGlobal(e.resource.typ()),
                keys: vec![signer],
                value: Some(value),
            })
        }
        MoveExpr::SignerAddress(e) => {
            let signer = lower_expr(builder, block, &e.expr);
            move_op(MoveOp::SignerAddress(signer))
        }
        MoveExpr::WriteRef(e) => {
            let reference = lower_expr(builder, block, &e.reference);
            let value = lower_expr(builder, block, &e.value);
            move_op(MoveOp::WriteRef(MoveWriteRefOp { reference, value }))
        }
    }
}

/// Lower an Anchor dialect expression. `Ok(e)` is a transparent wrapper.
fn lower_anchor_expr(builder: &mut CfgBuilder, block: BlockId, expr: &AnchorExpr) -> Lowered {
    let kind = match expr {
        AnchorExpr::AccountLoad(e) => {
            let account = lower_expr(builder, block, &e.expr);
            OpKind::Load(LoadOp { resource: Resource::AnchorAccount(account), keys: vec![] })
        }
        AnchorExpr::AccountLoadMut(e) => {
            let account = lower_expr(builder, block, &e.expr);
            anchor_op(AnchorOp::AccountLoadMut(account))
        }
        AnchorExpr::Cpi(e) => {
            let program = lower_expr(builder, block, &e.program);
            let parts = e.accounts.iter().chain(std::iter::once(&*e.data)).collect();
            let args = lower_exprs(builder, block, parts);
            external_call(ExternalKind::Cpi, Some(program), args, None)
        }
        AnchorExpr::FindProgramAddress(e) => {
            let seeds = lower_exprs(builder, block, e.seeds.iter().collect());
            let program_id = lower_expr(builder, block, &e.program_id);
            anchor_op(AnchorOp::FindProgramAddress(AnchorPdaOp { program_id, seeds }))
        }
        AnchorExpr::Ok(ok) => return Lowered::Value(lower_expr(builder, block, &ok.expr)),
        AnchorExpr::SignerKey(e) => {
            let account = lower_expr(builder, block, &e.expr);
            anchor_op(AnchorOp::SignerKey(account))
        }
        AnchorExpr::SystemTransfer(e) => {
            let args = lower_exprs(builder, block, vec![&e.from, &e.to]);
            let lamports = lower_expr(builder, block, &e.lamports);
            external_call(ExternalKind::SystemTransfer, None, args, Some(lamports))
        }
        AnchorExpr::TokenTransfer(e) => {
            let args = lower_exprs(builder, block, vec![&e.from, &e.to, &e.authority, &e.amount]);
            external_call(ExternalKind::TokenTransfer, None, args, None)
        }
    };
    Lowered::Kind(kind)
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

fn opaque(expr: &Expr, operands: Vec<OpRef>) -> OpKind {
    OpKind::Opaque { description: expr.to_string(), operands }
}

fn external_call(
    kind: ExternalKind,
    address: Option<OpRef>,
    args: Vec<OpRef>,
    value: Option<OpRef>,
) -> OpKind {
    let target = CallTarget::External(ExternalCallee { kind, address });
    OpKind::Call(CallOp { target, args, value })
}

fn evm_builtin(
    builder: &mut CfgBuilder,
    block: BlockId,
    builtin: EvmBuiltin,
    args: Vec<&Expr>,
) -> OpKind {
    let args = lower_exprs(builder, block, args);
    OpKind::Dialect(DialectOp::Evm(EvmOp::Builtin(EvmBuiltinOp { builtin, args })))
}

fn move_op(op: MoveOp) -> OpKind {
    OpKind::Dialect(DialectOp::Move(op))
}

fn anchor_op(op: AnchorOp) -> OpKind {
    OpKind::Dialect(DialectOp::Anchor(op))
}

/// Extract a name from an expression (for SSA naming).
fn expr_name(expr: &Expr) -> String {
    match expr {
        Expr::Var(v) => v.name.clone(),
        Expr::IndexAccess(idx) => format!("{}_idx", expr_name(&idx.base)),
        Expr::FieldAccess(field) => format!("{}_{}", expr_name(&field.base), field.field),
        _ => TMP_NAME.to_string(),
    }
}

// ========================================================================
// Tests
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bir::cfg::{Function, FunctionId};
    use crate::bir::module::Module;
    use crate::cir::lower::ssa;
    use crate::sir::evm::{EvmLowLevelCall, EvmMsgSender};
    use crate::sir::{
        AssignStmt, BinOp, BinOpExpr, ExprStmt, ForStmt, IfStmt, IndexAccessExpr,
        OverflowSemantics, ReturnStmt, StringLit, VarExpr, WhileStmt,
    };

    fn var(name: &str) -> Expr {
        Expr::Var(VarExpr::new(name.to_string(), Type::I256, None))
    }

    fn lit(value: bool) -> Expr {
        Expr::Lit(Lit::Bool(BoolLit::new(value, None)))
    }

    fn binop(op: BinOp, lhs: Expr, rhs: Expr) -> Expr {
        Expr::BinOp(BinOpExpr {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            overflow: OverflowSemantics::Checked,
            span: None,
        })
    }

    fn assign(name: &str, rhs: Expr) -> Stmt {
        Stmt::Assign(AssignStmt { lhs: var(name), rhs, span: None })
    }

    fn params(names: &[&str]) -> Vec<Param> {
        names.iter().map(|n| Param::new(n.to_string(), Type::I256)).collect()
    }

    /// `i = false; while (i < n) { i = i + true; }`
    fn counting_loop() -> Vec<Stmt> {
        let body = vec![assign("i", binop(BinOp::Add, var("i"), lit(true)))];
        let cond = binop(BinOp::Lt, var("i"), var("n"));
        vec![
            assign("i", lit(false)),
            Stmt::While(WhileStmt { cond, body, invariant: None, span: None }),
        ]
    }

    /// `x = a; y = a; if (c) { x = b; } return x + y;`
    fn conditional_update() -> Vec<Stmt> {
        let then_body = vec![assign("x", var("b"))];
        let sum = binop(BinOp::Add, var("x"), var("y"));
        vec![
            assign("x", var("a")),
            assign("y", var("a")),
            Stmt::If(IfStmt { cond: var("c"), then_body, else_body: None, span: None }),
            Stmt::Return(ReturnStmt { value: Some(sum), span: None }),
        ]
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

    fn scope_with_state(vars: &[&str]) -> ContractScope {
        ContractScope {
            contract: "C".to_string(),
            functions: HashSet::new(),
            storage_vars: vars.iter().map(|v| v.to_string()).collect(),
        }
    }

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
        let rhs = Expr::Lit(Lit::Bool(BoolLit::new(false, None)));
        Stmt::Assign(AssignStmt { lhs, rhs, span: None })
    }

    fn find_op<'b>(blocks: &'b [BasicBlock], id: OpRef) -> &'b Op {
        blocks.iter().flat_map(|b| &b.ops).find(|op| op.id == id.0).unwrap()
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
        let body = vec![
            Stmt::Expr(ExprStmt { expr: call, span: None }),
            assign_balance_of_sender(),
        ];
        let params = vec![
            Param::new("recipient".to_string(), Type::I256),
            Param::new("amount".to_string(), Type::I256),
        ];
        let blocks = build_cfg(&body, &params, &scope_with_state(&["balances"]));
        let ops = &blocks[0].ops;

        let call_pos = ops.iter().position(|op| matches!(op.kind, OpKind::Call(_))).unwrap();
        let OpKind::Call(call) = &ops[call_pos].kind else { unreachable!() };
        let CallTarget::External(callee) = &call.target else { panic!("expected external call") };
        assert_eq!(callee.kind, ExternalKind::Call);
        assert!(callee.address.is_some());
        assert_eq!(call.args.len(), 1);
        assert!(call.value.is_some());

        let store_pos = ops.iter().position(|op| matches!(op.kind, OpKind::Store(_))).unwrap();
        let OpKind::Store(store) = &ops[store_pos].kind else { unreachable!() };
        assert_eq!(store.resource, Resource::StateVar("balances".to_string()));
        assert_eq!(store.keys.len(), 1);
        assert!(matches!(find_op(&blocks, store.keys[0]).kind, OpKind::Env(EnvVar::Caller)));
        assert!(call_pos < store_pos, "state write must stay after the call");
    }

    #[test]
    fn test_local_shadowing_state_var_is_not_storage() {
        // A parameter named like a state variable shadows it.
        let params = vec![Param::new("balances".to_string(), Type::I256)];
        let body = vec![assign_balance_of_sender()];
        let blocks = build_cfg(&body, &params, &scope_with_state(&["balances"]));
        let has_storage_op = blocks
            .iter()
            .flat_map(|b| &b.ops)
            .any(|op| op.kind.storage_access().is_some());
        assert!(!has_storage_op);
    }

    #[test]
    fn test_loop_carried_variable_becomes_header_param() {
        let blocks = build_cfg(&counting_loop(), &params(&["n"]), &scope_with_state(&[]));

        // Only the loop header merges values of `i`.
        let [header] = param_blocks(&blocks)[..] else { panic!("expected one header") };
        assert_eq!(header.params.len(), 1);
        let i_param = OpRef(header.params[0].id);

        // The loop condition reads the header parameter.
        let Terminator::Branch { cond, .. } = &header.term else { panic!("expected branch") };
        let OpKind::BinOp { lhs, .. } = &find_op(&blocks, *cond).kind else { panic!() };
        assert_eq!(*lhs, i_param);

        // `i` enters as the initial constant and loops back as `i + 1`.
        let args = incoming(&blocks, header.id, 0);
        assert_eq!(args.len(), 2);
        assert!(args.iter().any(|op| matches!(op.kind, OpKind::Const(_))));
        assert!(args
            .iter()
            .any(|op| matches!(op.kind, OpKind::BinOp { lhs, .. } if lhs == i_param)));
    }

    #[test]
    fn test_merge_param_only_for_variables_changed_on_a_path() {
        let params = params(&["a", "b", "c"]);
        let blocks = build_cfg(&conditional_update(), &params, &scope_with_state(&[]));

        // `x` differs between the paths; `y` does not, so it gets no param.
        let [merge] = param_blocks(&blocks)[..] else { panic!("expected one merge block") };
        assert_eq!(merge.params.len(), 1);
        let x_param = OpRef(merge.params[0].id);
        let args = incoming(&blocks, merge.id, 0);
        let indices: HashSet<_> = args
            .iter()
            .map(|op| match op.kind {
                OpKind::Param { index } => index,
                _ => panic!("expected a function parameter"),
            })
            .collect();
        assert_eq!(indices, HashSet::from([0, 1]));

        // `return x + y` reads the merged `x` and the parameter `a` for `y`.
        let ret = blocks.iter().flat_map(|b| &b.ops).find_map(|op| match &op.kind {
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
        // for (i = false; i < n; i = i + true) { if (c) continue; if (d) break; }
        let jump_if = |cond: &str, stmt: Stmt| {
            Stmt::If(IfStmt { cond: var(cond), then_body: vec![stmt], else_body: None, span: None })
        };
        let body = vec![Stmt::For(ForStmt {
            init: Some(Box::new(assign("i", lit(false)))),
            cond: Some(binop(BinOp::Lt, var("i"), var("n"))),
            update: Some(Box::new(assign("i", binop(BinOp::Add, var("i"), lit(true))))),
            body: vec![jump_if("c", Stmt::Continue), jump_if("d", Stmt::Break)],
            invariant: None,
            span: None,
        })];
        let blocks = build_cfg(&body, &params(&["n", "c", "d"]), &scope_with_state(&[]));
        let transfers_into = |target: BlockId| {
            blocks.iter().flat_map(|b| b.term.successors()).filter(|s| *s == target).count()
        };

        // The loop exit is reached when the condition fails and on `break`.
        let header = blocks
            .iter()
            .find(|b| matches!(&b.term, Terminator::Branch { cond, .. }
                if matches!(find_op(&blocks, *cond).kind, OpKind::BinOp { op: BinOp::Lt, .. })))
            .unwrap();
        let Terminator::Branch { else_dest, .. } = &header.term else { unreachable!() };
        assert_eq!(transfers_into(else_dest.block), 2);

        // The latch runs the update; it is reached on `continue` and at the
        // end of the body, and loops back to the header.
        let latch = blocks
            .iter()
            .find(|b| b.ops.iter().any(|op| matches!(op.kind, OpKind::BinOp { op: BinOp::Add, .. })))
            .unwrap();
        assert_eq!(transfers_into(latch.id), 2);
        assert_eq!(latch.term.successors(), vec![header.id]);
    }

    #[test]
    fn test_ssa_output_passes_bir_verifier() {
        let mut module = Module::new("m".to_string());
        let bodies = [(counting_loop(), params(&["n"])), (conditional_update(), params(&["a", "b", "c"]))];
        for (index, (body, params)) in bodies.iter().enumerate() {
            let mut blocks = build_cfg(body, params, &scope_with_state(&[]));
            ssa::rename_to_ssa(&mut blocks);
            let mut func = Function::new(FunctionId(format!("C.f{index}")), true);
            func.blocks = blocks;
            module.functions.push(func);
        }
        assert!(crate::bir::verifier::verify(&module, false).is_ok());
    }
}
