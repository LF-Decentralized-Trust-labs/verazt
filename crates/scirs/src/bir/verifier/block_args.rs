//! BIR verifier pass: block_args
//!
//! Every control transfer passes exactly one argument per parameter of its
//! target block, and the entry block takes no parameters.

use crate::bir::cfg::{BlockId, Function};
use crate::bir::module::Module;
use crate::verify::VerifyError;
use std::collections::HashMap;

const PASS: &str = "bir::block_args";

pub fn check(module: &Module) -> Vec<VerifyError> {
    let mut errors = Vec::new();

    for func in &module.functions {
        check_function(func, &mut errors);
    }

    errors
}

fn check_function(func: &Function, errors: &mut Vec<VerifyError>) {
    if let Some(entry) = func.blocks.first() {
        if !entry.params.is_empty() {
            errors.push(VerifyError::new(
                PASS,
                format!("in {}, entry block {} has parameters", func.id, entry.id),
            ));
        }
    }

    let arity: HashMap<BlockId, usize> =
        func.blocks.iter().map(|b| (b.id, b.params.len())).collect();

    for block in &func.blocks {
        for call in block.term.block_calls() {
            // Unknown targets are reported by `cfg_well_formed`.
            let Some(&expected) = arity.get(&call.block) else {
                continue;
            };
            if call.args.len() != expected {
                errors.push(VerifyError::new(
                    PASS,
                    format!(
                        "in {}, block {} passes {} argument(s) to {}, which takes {expected}",
                        func.id,
                        block.id,
                        call.args.len(),
                        call.block
                    ),
                ));
            }
        }
    }
}
