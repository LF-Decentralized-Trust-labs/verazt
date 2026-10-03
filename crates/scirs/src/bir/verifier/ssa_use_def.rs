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
    span: Option<&crate::sir::Loc>,
    errors: &mut Vec<VerifyError>,
) {
    if !defined.contains(&op_ref.0) {
        let mut err = VerifyError::new(
            PASS,
            format!("OpRef {} references undefined OpId {}", op_ref, op_ref.0),
        );
        if let Some(span) = span {
            err = err.with_span(span.clone());
        }
        errors.push(err);
    }
}

fn check_op_uses(op: &Op, defined: &HashSet<OpId>, errors: &mut Vec<VerifyError>) {
    for operand in op.kind.operands() {
        check_ref(&operand, defined, op.span.as_ref(), errors);
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
        let span = block.ops.last().and_then(|op| op.span.as_ref());
        check_ref(cond, defined, span, errors);
    }
}
