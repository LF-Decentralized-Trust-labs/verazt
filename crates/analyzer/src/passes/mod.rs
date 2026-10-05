//! Analysis passes organised by IR layer.
//!
//! - `base/` — abstract infrastructure: traits, metadata
//! - `bir/` — passes operating on the BIR (analysis IR)

pub mod base;
pub mod bir;

use crate::pass_manager::PassRegistry;

/// Register all built-in analysis passes.
pub fn register_all_passes(registry: &mut PassRegistry) {
    registry.register::<bir::DefUsePass>();
    registry.register::<bir::DominancePass>();
    registry.register::<bir::FunctionEffectsPass>();
    registry.register::<bir::ICFGPass>();
    registry.register::<bir::IntervalPass>();
    registry.register::<bir::TaintPass>();
}
