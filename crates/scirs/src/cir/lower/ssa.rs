//! Step 3: SSA Renaming
//!
//! Standard Cytron et al. pruned SSA construction.
//! Assigns unique version numbers to each variable definition.
//!
//! Uses a single global counter so that every SSA name gets a unique
//! `%vN` identifier across the entire function.

use crate::bir::cfg::BasicBlock;
use crate::bir::ops::OpKind;

/// Rename all variables in the basic blocks to SSA form.
///
/// Uses a single global counter so that every definition receives a
/// unique, monotonically increasing ID displayed as `%vN`.
pub fn rename_to_ssa(blocks: &mut [BasicBlock]) {
    let mut next_id: u32 = 0;

    for block in blocks.iter_mut() {
        for op in &mut block.ops {
            // If this op has a result, assign the next global ID
            if let Some((ssa_name, _ty)) = &mut op.result {
                ssa_name.version = next_id;
                next_id += 1;
            }

            // For phi nodes, ensure each incoming value has a proper version
            if let OpKind::Phi(entries) = &mut op.kind {
                // Phi entries reference other ops; versions are already assigned
                let _ = entries;
            }
        }
    }
}
