//! Visit pattern for BIR — read-only traversal.

use crate::bir::cfg::{BasicBlock, BlockCall, Function, Terminator};
use crate::bir::module::Module;
use crate::bir::ops::*;

/// Trait implementing the visit design pattern for BIR.
pub trait Visit<'a> {
    // ── Module / Function ───────────────────────────
    fn visit_module(&mut self, module: &'a Module) {
        default::visit_module(self, module)
    }
    fn visit_function(&mut self, func: &'a Function) {
        default::visit_function(self, func)
    }
    fn visit_basic_block(&mut self, block: &'a BasicBlock) {
        default::visit_basic_block(self, block)
    }

    // ── Ops ─────────────────────────────────────────
    fn visit_op(&mut self, op: &'a Op) {
        default::visit_op(self, op)
    }
    fn visit_const_op(&mut self, _lit: &'a crate::sir::Lit) {}
    fn visit_binop_op(&mut self, _op: &'a crate::sir::BinOp, _lhs: &'a OpRef, _rhs: &'a OpRef) {}
    fn visit_unop_op(&mut self, _op: &'a crate::sir::UnOp, _operand: &'a OpRef) {}
    fn visit_assert_op(&mut self, _cond: &'a OpRef) {}
    fn visit_return_op(&mut self, _vals: &'a [OpRef]) {}
    fn visit_param_op(&mut self, _index: &'a ParamIndex) {}
    fn visit_expr_stmt_op(&mut self, _expr: &'a OpRef) {}
    fn visit_load_op(&mut self, _op: &'a LoadOp) {}
    fn visit_store_op(&mut self, _op: &'a StoreOp) {}
    fn visit_call_op(&mut self, _op: &'a CallOp) {}
    fn visit_env_op(&mut self, _var: &'a EnvVar) {}
    fn visit_emit_op(&mut self, _op: &'a EmitOp) {}
    fn visit_dialect_op(&mut self, _op: &'a DialectOp) {}
    fn visit_opaque_op(&mut self, _description: &'a str, _operands: &'a [OpRef]) {}

    // ── Terminators ─────────────────────────────────
    fn visit_terminator(&mut self, term: &'a Terminator) {
        default::visit_terminator(self, term)
    }
    fn visit_branch_term(
        &mut self,
        _cond: &'a OpRef,
        _then_dest: &'a BlockCall,
        _else_dest: &'a BlockCall,
    ) {
    }
    fn visit_jump_term(&mut self, _dest: &'a BlockCall) {}
    fn visit_txn_exit_term(&mut self, _reverted: bool) {}
}

/// Default implementations for the BIR Visit trait.
pub mod default {
    use super::Visit;
    use crate::bir::cfg::{BasicBlock, Function, Terminator};
    use crate::bir::module::Module;
    use crate::bir::ops::*;

    pub fn visit_module<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, module: &'a Module) {
        for f in &module.functions {
            visitor.visit_function(f)
        }
    }

    pub fn visit_function<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, func: &'a Function) {
        for block in &func.blocks {
            visitor.visit_basic_block(block)
        }
    }

    pub fn visit_basic_block<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, block: &'a BasicBlock) {
        for op in &block.ops {
            visitor.visit_op(op)
        }
        visitor.visit_terminator(&block.term)
    }

    pub fn visit_op<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, op: &'a Op) {
        match &op.kind {
            OpKind::Const(lit) => visitor.visit_const_op(lit),
            OpKind::BinOp { op: binop, lhs, rhs, .. } => visitor.visit_binop_op(binop, lhs, rhs),
            OpKind::UnOp { op: unop, operand } => visitor.visit_unop_op(unop, operand),
            OpKind::Assert { cond } => visitor.visit_assert_op(cond),
            OpKind::Return(vals) => visitor.visit_return_op(vals),
            OpKind::Param { index } => visitor.visit_param_op(index),
            OpKind::ExprStmt { expr } => visitor.visit_expr_stmt_op(expr),
            OpKind::Load(load) => visitor.visit_load_op(load),
            OpKind::Store(store) => visitor.visit_store_op(store),
            OpKind::Call(call) => visitor.visit_call_op(call),
            OpKind::Env(var) => visitor.visit_env_op(var),
            OpKind::Emit(emit) => visitor.visit_emit_op(emit),
            OpKind::Dialect(dialect) => visitor.visit_dialect_op(dialect),
            OpKind::Symbol { .. } => {}
            OpKind::Opaque { description, operands } => {
                visitor.visit_opaque_op(description, operands)
            }
        }
    }

    pub fn visit_terminator<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, term: &'a Terminator) {
        match term {
            Terminator::Branch { cond, then_dest, else_dest } => {
                visitor.visit_branch_term(cond, then_dest, else_dest)
            }
            Terminator::Jump(target) => visitor.visit_jump_term(target),
            Terminator::TxnExit { reverted } => visitor.visit_txn_exit_term(*reverted),
            Terminator::Unreachable => {}
        }
    }
}
