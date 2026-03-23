//! BIR → FIR lowering.
//!
//! Lifts each BIR basic block into a FIR tail-recursive function.

pub mod lift_blocks;

use crate::bir;
use crate::fir;

/// Lower a BIR Module into a FIR Module.
pub fn lower_module(bir_module: &bir::Module) -> fir::Module {
    let mut fir_module = fir::Module::new(bir_module.source_module_id.clone());

    for func in &bir_module.functions {
        let fir_functions = lift_blocks::lift_function(func);
        fir_module.functions.extend(fir_functions);
    }

    fir_module
}
