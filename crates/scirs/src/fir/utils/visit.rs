//! Visit pattern for FIR — read-only traversal.

use crate::bir::ops::*;
use crate::fir::ops::*;

/// Trait implementing the visit design pattern for FIR.
pub trait Visit<'a> {
    // ── Module / Function ───────────────────────────
    fn visit_module(&mut self, module: &'a Module) {
        default::visit_module(self, module)
    }
    fn visit_function(&mut self, func: &'a Function) {
        default::visit_function(self, func)
    }

    // ── Ops (reused from BIR) ───────────────────────
    fn visit_op(&mut self, op: &'a Op) {
        default::visit_op(self, op)
    }
    fn visit_const_op(&mut self, _lit: &'a crate::sir::Lit) {}
    fn visit_binop_op(&mut self, _op: &'a crate::sir::BinOp, _lhs: &'a OpRef, _rhs: &'a OpRef) {}
    fn visit_unop_op(&mut self, _op: &'a crate::sir::UnOp, _operand: &'a OpRef) {}
    fn visit_phi_op(&mut self, _entries: &'a [(crate::bir::cfg::BlockId, OpRef)]) {}
    fn visit_assert_op(&mut self, _cond: &'a OpRef) {}
    fn visit_return_op(&mut self, _vals: &'a [OpRef]) {}
    fn visit_param_op(&mut self, _index: &'a ParamIndex) {}
    fn visit_expr_stmt_op(&mut self, _expr: &'a OpRef) {}
    fn visit_storage_op(&mut self, _op: &'a StorageDialectOp) {}
    fn visit_call_op(&mut self, _op: &'a CallDialectOp) {}
    fn visit_taint_src_op(&mut self, _op: &'a TaintSourceOp) {}
    fn visit_taint_snk_op(&mut self, _op: &'a TaintSinkOp) {}
    fn visit_opaque_op(&mut self, _description: &'a str) {}

    // ── Terminators (FIR-specific) ──────────────────
    fn visit_terminator(&mut self, term: &'a Terminator) {
        default::visit_terminator(self, term)
    }
    fn visit_tail_call_term(&mut self, _tc: &'a TailCallData) {}
    fn visit_branch_term(
        &mut self,
        _cond: &'a OpRef,
        _then_call: &'a TailCallData,
        _else_call: &'a TailCallData,
    ) {
    }
    fn visit_return_term(&mut self, _vals: &'a [OpRef]) {}
    fn visit_revert_term(&mut self) {}
}

/// Default implementations for the FIR Visit trait.
pub mod default {
    use super::Visit;
    use crate::bir::ops::*;
    use crate::fir::ops::*;

    pub fn visit_module<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, module: &'a Module) {
        for f in &module.functions {
            visitor.visit_function(f)
        }
    }

    pub fn visit_function<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, func: &'a Function) {
        for op in &func.body {
            visitor.visit_op(op)
        }
        visitor.visit_terminator(&func.term)
    }

    pub fn visit_op<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, op: &'a Op) {
        match &op.kind {
            OpKind::Const(lit) => visitor.visit_const_op(lit),
            OpKind::BinOp { op: binop, lhs, rhs, .. } => visitor.visit_binop_op(binop, lhs, rhs),
            OpKind::UnOp { op: unop, operand } => visitor.visit_unop_op(unop, operand),
            OpKind::Phi(entries) => visitor.visit_phi_op(entries),
            OpKind::Assert { cond } => visitor.visit_assert_op(cond),
            OpKind::Return(vals) => visitor.visit_return_op(vals),
            OpKind::Param { index } => visitor.visit_param_op(index),
            OpKind::ExprStmt { expr } => visitor.visit_expr_stmt_op(expr),
            OpKind::Storage(s) => visitor.visit_storage_op(s),
            OpKind::Call(c) => visitor.visit_call_op(c),
            OpKind::TaintSrc(t) => visitor.visit_taint_src_op(t),
            OpKind::TaintSnk(t) => visitor.visit_taint_snk_op(t),
            OpKind::PseudoValue { .. } => {}
            OpKind::Opaque { description } => visitor.visit_opaque_op(description),
        }
    }

    pub fn visit_terminator<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, term: &'a Terminator) {
        match term {
            Terminator::TailCall(tc) => visitor.visit_tail_call_term(tc),
            Terminator::Branch { cond, then_call, else_call } => {
                visitor.visit_branch_term(cond, then_call, else_call)
            }
            Terminator::Return(vals) => visitor.visit_return_term(vals),
            Terminator::Revert => visitor.visit_revert_term(),
            Terminator::Unreachable => {}
        }
    }
}
