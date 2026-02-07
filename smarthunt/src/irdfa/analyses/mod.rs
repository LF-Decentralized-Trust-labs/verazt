//! Built-in data flow analyses
//!
//! This module contains implementations of common data flow analyses:
//!
//! - **Reaching Definitions**: Which definitions reach each program point
//! - **Live Variables**: Which variables are live at each program point
//! - **Def-Use Chains**: Tracks definition-use relationships
//! - **Taint Analysis**: Tracks data flow from untrusted sources to sensitive operations
//! - **State Mutation**: Tracks modifications to contract state

pub mod reaching_defs;
pub mod liveness;
pub mod def_use;
pub mod taint;
pub mod state_mutation;

pub use reaching_defs::{Definition, ReachingDefsPass, ReachingDefsTransfer};
pub use liveness::{LivenessPass, LiveVarsTransfer};
pub use def_use::{DefUseChainsPass, Use};
pub use taint::{TaintAnalysisPass, TaintSource, TaintSink};
pub use state_mutation::{StateMutationPass, StateAccess, AccessKind};
