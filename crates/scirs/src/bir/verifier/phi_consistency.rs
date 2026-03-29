//! BIR verifier pass: phi_consistency
//!
//! Every Phi node has one entry per predecessor block; no phi in entry
//! block without predecessors.

use crate::bir::cfg::{BlockId, Function, Terminator};
use crate::bir::module::Module;
use crate::bir::ops::*;
use crate::verify::VerifyError;
use std::collections::{HashMap, HashSet};

const PASS: &str = "bir::phi_consistency";

pub fn check(module: &Module) -> Vec<VerifyError> {
    let mut errors = Vec::new();

    for func in &module.functions {
        check_function(func, &mut errors);
    }

    errors
}

fn check_function(func: &Function, errors: &mut Vec<VerifyError>) {
    // Build predecessor map: block_id -> set of predecessor block_ids
    let mut preds: HashMap<usize, HashSet<usize>> = HashMap::new();
    for block in &func.blocks {
        preds.entry(block.id.0).or_default();
    }

    for block in &func.blocks {
        match &block.term {
            Terminator::Branch { then_bb, else_bb, .. } => {
                preds.entry(then_bb.0).or_default().insert(block.id.0);
                preds.entry(else_bb.0).or_default().insert(block.id.0);
            }
            Terminator::Jump(target) => {
                preds.entry(target.0).or_default().insert(block.id.0);
            }
            Terminator::TxnExit { .. } | Terminator::Unreachable => {}
        }
    }

    for block in &func.blocks {
        let block_preds = preds.get(&block.id.0).cloned().unwrap_or_default();

        for op in &block.ops {
            if let OpKind::Phi(entries) = &op.kind {
                // No phi in entry block
                if block.id.0 == 0 && block_preds.is_empty() {
                    let mut err = VerifyError::new(
                        PASS,
                        format!(
                            "in {}, phi {} in entry block %bb0 with no predecessors",
                            func.id, op.id
                        ),
                    );
                    if let Some(span) = &op.span {
                        err = err.with_span(span.clone());
                    }
                    errors.push(err);
                }

                // Phi entries should match predecessors
                let phi_blocks: HashSet<usize> =
                    entries.iter().map(|(BlockId(id), _)| *id).collect();

                for pred in &block_preds {
                    if !phi_blocks.contains(pred) {
                        errors.push(VerifyError::new(
                            PASS,
                            format!(
                                "in {}, phi {} in block {} missing entry for predecessor %bb{pred}",
                                func.id, op.id, block.id
                            ),
                        ));
                    }
                }

                for (BlockId(phi_bb), _) in entries {
                    if !block_preds.contains(phi_bb) {
                        errors.push(VerifyError::new(
                            PASS,
                            format!(
                                "in {}, phi {} in block {} has entry for %bb{phi_bb} which is not a predecessor",
                                func.id, op.id, block.id
                            ),
                        ));
                    }
                }
            }
        }
    }
}
