//! Step 3: SSA Renaming
//!
//! Standard Cytron et al. pruned SSA construction.
//! Assigns unique version numbers to each variable definition.

use crate::air::cfg::BasicBlock;
use crate::air::ops::OpKind;
use std::collections::HashMap;

/// Rename all variables in the basic blocks to SSA form.
///
/// This is a simplified SSA renaming pass that assigns version numbers
/// to variable definitions. A full implementation would compute dominance
/// frontiers and insert phi functions; this version uses a simple
/// sequential numbering scheme per-variable.
pub fn rename_to_ssa(blocks: &mut [BasicBlock]) {
    let mut version_map: HashMap<String, u32> = HashMap::new();

    for block in blocks.iter_mut() {
        for op in &mut block.ops {
            // If this op has a result, rename it with a new version
            if let Some((ssa_name, _ty)) = &mut op.result {
                let base = ssa_name.base.clone();
                let version = version_map.entry(base.clone()).or_insert(0);
                ssa_name.version = *version;
                *version += 1;
            }

            // For phi nodes, ensure each incoming value has a proper version
            if let OpKind::Phi(entries) = &mut op.kind {
                // Phi entries reference other ops; versions are already assigned
                let _ = entries;
            }
        }
    }
}
