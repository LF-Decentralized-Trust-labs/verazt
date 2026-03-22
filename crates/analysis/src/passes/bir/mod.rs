//! BIR-layer analysis passes.

pub mod def_use;
pub mod dominance;
pub mod icfg;
pub mod interval;
pub mod taint;
pub mod taint_propagation;

pub use def_use::{DefUseArtifact, DefUsePass};
pub use dominance::{DominanceArtifact, DominancePass};
pub use icfg::{ICFGArtifact, ICFGPass};
pub use interval::{Interval, IntervalArtifact, IntervalPass};
pub use taint::{TaintArtifact, TaintPass};
pub use taint_propagation::TaintPropagationPass;
