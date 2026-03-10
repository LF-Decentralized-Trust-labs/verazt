//! Analysis Framework
//!
//! # Architecture
//!
//! The analysis framework provides:
//! - **Pass Infrastructure** (`passes/base/`): Base traits, metadata for
//!   passes.
//! - **Pipeline** (`pipeline/`): PassManager, scheduler, executor, dependency
//!   graph.
//! - **Passes** (`passes/`): Concrete analysis passes organised by IR layer
//!   (SIR, AIR, VIR).
//! - **Frameworks** (`frameworks/`): Reusable analysis infrastructure —
//!   dataflow analysis (`dfa/`) and control-flow analysis (`cfa/`).
//! - **AnalysisContext**: Central storage for SIR, AIR, and analysis artifacts.
//!
//! # Usage
//!
//! ```ignore
//! use analysis::{PassManager, AnalysisContext, PassManagerConfig};
//!
//! let mut manager = PassManager::new(PassManagerConfig::default());
//! // Register passes...
//!
//! let mut context = AnalysisContext::new(sir_modules, Default::default());
//! let report = manager.run(&mut context)?;
//! ```

// Core modules
pub mod context;
pub mod pipeline;

// Analysis passes, organised by IR layer
pub mod passes;

/// Convenience alias so that `analysis::pass::*` keeps working for
/// downstream crates (e.g. scanner) that have not migrated yet.
pub mod pass {
    pub use crate::passes::base::*;
    pub mod meta {
        pub use crate::passes::base::meta::*;
    }
    pub mod traits {
        pub use crate::passes::base::traits::*;
    }
}

// Reusable analysis infrastructure
pub mod frameworks;

// Re-exports for convenient access
pub use context::{AnalysisConfig, AnalysisContext, AnalysisStats, ArtifactKey, InputLanguage};
pub use passes::base::meta::{PassLevel, PassRepresentation};
pub use passes::base::{AnalysisPass, Pass, PassError, PassExecutionInfo, PassResult};
pub use pipeline::{
    AnalysisReport, DependencyGraph, ExecutionResult, ExecutorConfig, PassExecutor, PassManager,
    PassManagerConfig, PassScheduler,
};

// Re-export concrete passes
pub use passes::air::TaintPropagationPass;

// Re-export DFA framework
pub use frameworks::dfa::analyses::{
    DefUseChainsPass, Definition, LiveVarsTransfer, LivenessPass, ReachingDefsPass,
    ReachingDefsTransfer, StateMutationPass, TaintAnalysisPass, TaintSource, Use,
};
pub use frameworks::dfa::{
    DataFlowResult, DataFlowSolver, Direction, FlatLattice, Lattice, MapLattice, PowerSetLattice,
    ProductLattice, Transfer, VarId, VarScope,
};

// Re-export CFA framework
pub use frameworks::cfa::{
    callgraph::SirCallGraph,
    domtree::{DomTree, PostDomTree},
    loops::{LoopInfo, NaturalLoop},
    reachability::{self, ReachabilitySet},
};
