//! BIR-layer analysis passes.

pub mod def_use;
pub mod dominance;
pub mod function_effects;
pub mod icfg;
pub mod interval;
pub mod taint;

pub use def_use::{DefUseArtifact, DefUsePass};
pub use dominance::{DominanceArtifact, DominancePass};
pub use function_effects::{FunctionEffects, FunctionEffectsArtifact, FunctionEffectsPass};
pub use icfg::{ICFGArtifact, ICFGPass};
pub use interval::{Interval, IntervalArtifact, IntervalPass};
pub use taint::{TaintArtifact, TaintPass};
