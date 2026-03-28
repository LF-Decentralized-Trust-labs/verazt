//! Compatibility shim — re-exports from the flattened module layout.
//!
//! All modules formerly under `analysis::` now live at the crate root:
//! - `crate::context`
//! - `crate::pass_manager`
//! - `crate::passes`
//! - `crate::frameworks`

/// Convenience alias so that `analysis::pass::*` keeps working for
/// downstream crates that have not migrated yet.
pub mod pass {
    pub use crate::passes::base::*;
    pub mod meta {
        pub use crate::passes::base::meta::*;
    }
    pub mod traits {
        pub use crate::passes::base::traits::*;
    }
}

// Re-exports for convenient access
pub use crate::context::{
    AnalysisConfig, AnalysisContext, AnalysisStats, ArtifactKey, InputLanguage,
};
pub use crate::passes::base::meta::{PassLevel, PassRepresentation};
pub use crate::passes::base::{AnalysisPass, Pass, PassError, PassExecutionInfo, PassResult};
pub use crate::pass_manager::{
    AnalysisReport, DependencyGraph, ExecutionResult, ExecutorConfig, PassExecutor, PassManager,
    PassManagerConfig, PassScheduler,
};

// Re-export concrete passes
pub use crate::passes::bir::{
    DefUseArtifact, DefUsePass, DominanceArtifact, DominancePass, ICFGArtifact, ICFGPass,
    Interval, IntervalArtifact, IntervalPass, TaintArtifact, TaintPass, TaintPropagationPass,
};
pub use crate::passes::sir::{WriteSetArtifact, WriteSetPass};

// Re-export DFA framework
pub use crate::frameworks::dfa::analyses::{
    DefUseChainsPass, Definition, LiveVarsTransfer, LivenessPass, ReachingDefsPass,
    ReachingDefsTransfer, StateMutationPass, TaintAnalysisPass, TaintSource, Use,
};
pub use crate::frameworks::dfa::{
    DataFlowResult, DataFlowSolver, Direction, FlatLattice, Lattice, MapLattice, PowerSetLattice,
    ProductLattice, Transfer, VarId, VarScope,
};

// Re-export CFA framework
pub use crate::frameworks::cfa::{
    callgraph::SirCallGraph,
    domtree::{DomTree, PostDomTree},
    loops::{LoopInfo, NaturalLoop},
    reachability::{self, ReachabilitySet},
};
