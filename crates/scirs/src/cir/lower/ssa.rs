//! Step 2: SSA Name Numbering
//!
//! SSA form itself is built during CFG construction (`cfg.rs`). This step
//! only gives every definition (block parameter or op result) a unique,
//! monotonically increasing version, displayed as `%vN`.

use crate::bir::cfg::BasicBlock;

/// Number all SSA definitions in the basic blocks.
pub fn rename_to_ssa(blocks: &mut [BasicBlock]) {
    let mut next_id: u32 = 0;

    for block in blocks.iter_mut() {
        for param in &mut block.params {
            param.name.version = next_id;
            next_id += 1;
        }
        for op in &mut block.ops {
            if let Some((ssa_name, _ty)) = &mut op.result {
                ssa_name.version = next_id;
                next_id += 1;
            }
        }
    }
}
