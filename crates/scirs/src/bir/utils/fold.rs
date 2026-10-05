//! Fold pattern for BIR — accumulating traversal.

use crate::bir::cfg::{BasicBlock, Function, Terminator};
use crate::bir::module::Module;
use crate::bir::ops::*;

/// Trait implementing the fold design pattern for BIR.
/// Traverses the BIR tree while accumulating a result of type `T`.
pub trait Fold<'a, T: Default> {
    // ── Module / Function ───────────────────────────
    fn fold_module(&mut self, module: &'a Module) -> T {
        default::fold_module(self, module)
    }
    fn fold_function(&mut self, func: &'a Function) -> T {
        default::fold_function(self, func)
    }
    fn fold_basic_block(&mut self, block: &'a BasicBlock) -> T {
        default::fold_basic_block(self, block)
    }

    // ── Ops ─────────────────────────────────────────
    fn fold_op(&mut self, op: &'a Op) -> T {
        default::fold_op(self, op)
    }
    fn fold_const_op(&mut self, _lit: &'a crate::sir::Lit) -> T {
        T::default()
    }
    fn fold_binop_op(
        &mut self,
        _op: &'a crate::sir::BinOp,
        _lhs: &'a OpRef,
        _rhs: &'a OpRef,
    ) -> T {
        T::default()
    }
    fn fold_unop_op(&mut self, _op: &'a crate::sir::UnOp, _operand: &'a OpRef) -> T {
        T::default()
    }
    fn fold_assert_op(&mut self, _cond: &'a OpRef) -> T {
        T::default()
    }
    fn fold_return_op(&mut self, _vals: &'a [OpRef]) -> T {
        T::default()
    }
    fn fold_param_op(&mut self, _index: &'a ParamIndex) -> T {
        T::default()
    }
    fn fold_expr_stmt_op(&mut self, _expr: &'a OpRef) -> T {
        T::default()
    }
    fn fold_load_op(&mut self, _op: &'a LoadOp) -> T {
        T::default()
    }
    fn fold_store_op(&mut self, _op: &'a StoreOp) -> T {
        T::default()
    }
    fn fold_call_op(&mut self, _op: &'a CallOp) -> T {
        T::default()
    }
    fn fold_env_op(&mut self, _var: &'a EnvVar) -> T {
        T::default()
    }
    fn fold_emit_op(&mut self, _op: &'a EmitOp) -> T {
        T::default()
    }
    fn fold_dialect_op(&mut self, _op: &'a DialectOp) -> T {
        T::default()
    }
    fn fold_opaque_op(&mut self, _description: &'a str, _operands: &'a [OpRef]) -> T {
        T::default()
    }

    // ── Terminators ─────────────────────────────────
    fn fold_terminator(&mut self, term: &'a Terminator) -> T {
        default::fold_terminator(self, term)
    }

    /// Combine two folded values.
    fn combine(&mut self, a: T, _b: T) -> T {
        a
    }
}

/// Default implementations for the BIR Fold trait.
pub mod default {
    use super::Fold;
    use crate::bir::cfg::{BasicBlock, Function, Terminator};
    use crate::bir::module::Module;
    use crate::bir::ops::*;

    pub fn fold_module<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        module: &'a Module,
    ) -> T {
        let mut result = T::default();
        for func in &module.functions {
            let r = folder.fold_function(func);
            result = folder.combine(result, r);
        }
        result
    }

    pub fn fold_function<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        func: &'a Function,
    ) -> T {
        let mut result = T::default();
        for block in &func.blocks {
            let r = folder.fold_basic_block(block);
            result = folder.combine(result, r);
        }
        result
    }

    pub fn fold_basic_block<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        block: &'a BasicBlock,
    ) -> T {
        let mut result = T::default();
        for op in &block.ops {
            let r = folder.fold_op(op);
            result = folder.combine(result, r);
        }
        let t = folder.fold_terminator(&block.term);
        folder.combine(result, t)
    }

    pub fn fold_op<'a, T: Default, F: Fold<'a, T> + ?Sized>(folder: &mut F, op: &'a Op) -> T {
        match &op.kind {
            OpKind::Const(lit) => folder.fold_const_op(lit),
            OpKind::BinOp { op: binop, lhs, rhs, .. } => folder.fold_binop_op(binop, lhs, rhs),
            OpKind::UnOp { op: unop, operand } => folder.fold_unop_op(unop, operand),
            OpKind::Assert { cond } => folder.fold_assert_op(cond),
            OpKind::Return(vals) => folder.fold_return_op(vals),
            OpKind::Param { index } => folder.fold_param_op(index),
            OpKind::ExprStmt { expr } => folder.fold_expr_stmt_op(expr),
            OpKind::Load(load) => folder.fold_load_op(load),
            OpKind::Store(store) => folder.fold_store_op(store),
            OpKind::Call(call) => folder.fold_call_op(call),
            OpKind::Env(var) => folder.fold_env_op(var),
            OpKind::Emit(emit) => folder.fold_emit_op(emit),
            OpKind::Dialect(dialect) => folder.fold_dialect_op(dialect),
            OpKind::Symbol { .. } => T::default(),
            OpKind::Opaque { description, operands } => {
                folder.fold_opaque_op(description, operands)
            }
        }
    }

    pub fn fold_terminator<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        _folder: &mut F,
        _term: &'a Terminator,
    ) -> T {
        T::default()
    }
}
