//! BIR verifier pass: ssa_use_def
//!
//! Every `OpRef` references a previously defined `OpId`.

use crate::bir::cfg::{BasicBlock, Function, Terminator};
use crate::bir::module::Module;
use crate::bir::ops::*;
use crate::verify::VerifyError;
use std::collections::HashSet;

const PASS: &str = "bir::ssa_use_def";

pub fn check(module: &Module) -> Vec<VerifyError> {
    let mut errors = Vec::new();

    for func in &module.functions {
        check_function(func, &mut errors);
    }

    errors
}

fn check_function(func: &Function, errors: &mut Vec<VerifyError>) {
    // Collect all defined OpIds in this function.
    let mut defined: HashSet<OpId> = HashSet::new();
    for block in &func.blocks {
        for op in &block.ops {
            defined.insert(op.id);
        }
    }

    // Check all uses reference a defined OpId.
    for block in &func.blocks {
        for op in &block.ops {
            check_op_uses(op, &defined, errors);
        }
        check_term_uses(&block.term, &defined, block, errors);
    }
}

fn check_ref(
    op_ref: &OpRef,
    defined: &HashSet<OpId>,
    span: Option<crate::sir::Span>,
    errors: &mut Vec<VerifyError>,
) {
    if !defined.contains(&op_ref.0) {
        let mut err = VerifyError::new(
            PASS,
            format!("OpRef {} references undefined OpId {}", op_ref, op_ref.0),
        );
        if let Some(span) = span {
            err = err.with_span(span);
        }
        errors.push(err);
    }
}

fn check_op_uses(op: &Op, defined: &HashSet<OpId>, errors: &mut Vec<VerifyError>) {
    match &op.kind {
        OpKind::BinOp { lhs, rhs, .. } => {
            check_ref(lhs, defined, op.span, errors);
            check_ref(rhs, defined, op.span, errors);
        }
        OpKind::UnOp { operand, .. } => {
            check_ref(operand, defined, op.span, errors);
        }
        OpKind::Phi(entries) => {
            for (_block, r) in entries {
                check_ref(r, defined, op.span, errors);
            }
        }
        OpKind::Assert { cond } => {
            check_ref(cond, defined, op.span, errors);
        }
        OpKind::Return(vals) => {
            for r in vals {
                check_ref(r, defined, op.span, errors);
            }
        }
        OpKind::ExprStmt { expr } => {
            check_ref(expr, defined, op.span, errors);
        }
        OpKind::Storage(s) => {
            if let Some(k) = &s.key_operand {
                check_ref(k, defined, op.span, errors);
            }
            if let Some(v) = &s.value_operand {
                check_ref(v, defined, op.span, errors);
            }
        }
        OpKind::Call(c) => {
            for arg in &c.args {
                check_ref(arg, defined, op.span, errors);
            }
        }
        OpKind::Const(_)
        | OpKind::Param { .. }
        | OpKind::TaintSrc(_)
        | OpKind::TaintSnk(_)
        | OpKind::PseudoValue { .. }
        | OpKind::Opaque { .. } => {}
    }
}

fn check_term_uses(
    term: &Terminator,
    defined: &HashSet<OpId>,
    block: &BasicBlock,
    errors: &mut Vec<VerifyError>,
) {
    if let Terminator::Branch { cond, .. } = term {
        // Use the first op's span as a rough location
        let span = block.ops.last().and_then(|op| op.span);
        check_ref(cond, defined, span, errors);
    }
}
