//! BIR verifier pass: ssa_single_def
//!
//! Every `SsaName` is defined (assigned) exactly once across all blocks.

use crate::bir::cfg::{BasicBlock, Function};
use crate::bir::module::Module;
use crate::bir::ops::*;
use crate::verify::VerifyError;
use std::collections::HashMap;

const PASS: &str = "bir::ssa_single_def";

pub fn check(module: &Module) -> Vec<VerifyError> {
    let mut errors = Vec::new();

    for func in &module.functions {
        check_function(func, &mut errors);
    }

    errors
}

fn check_function(func: &Function, errors: &mut Vec<VerifyError>) {
    // Map from SsaName to (block_id, op_id) of first definition.
    let mut defs: HashMap<SsaName, (usize, OpId)> = HashMap::new();

    for block in &func.blocks {
        collect_defs(block, &mut defs, errors);
    }
}

fn collect_defs(
    block: &BasicBlock,
    defs: &mut HashMap<SsaName, (usize, OpId)>,
    errors: &mut Vec<VerifyError>,
) {
    for op in &block.ops {
        if let Some((name, _ty)) = &op.result {
            if let Some((_prev_block, prev_op)) = defs.get(name) {
                let mut err = VerifyError::new(
                    PASS,
                    format!(
                        "SSA name `{name}` defined multiple times: first at {prev_op}, again at {}",
                        op.id
                    ),
                );
                if let Some(span) = &op.span {
                    err = err.with_span(span.clone());
                }
                errors.push(err);
            } else {
                defs.insert(name.clone(), (block.id.0, op.id));
            }
        }
    }
}
