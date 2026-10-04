//! Execution Pipeline
//!
//! This module defines **how passes are run**: registration, scheduling,
//! execution, and dependency resolution.
//!
//! ## Responsibility boundaries
//!
//! - **`manager`**: the single owner of the registered passes; entry point
//!   for callers; delegates to scheduler then executor; produces
//!   `PassRunReport`.
//! - **`scheduler`**: pure function from the registered passes to an
//!   `ExecutionSchedule`; must not mutate `AnalysisContext`.
//! - **`executor`**: borrows the manager's passes, takes `ExecutionSchedule`
//!   + `AnalysisContext`; drives execution and timing.
//! - **`dependency`**: dependency graph, topological sort ordered by pass
//!   name.
//! - **`registry`**: constructors of the passes that may be scheduled.

pub mod dependency;
pub mod executor;
pub mod manager;
pub mod registry;
pub mod scheduler;

pub use dependency::DependencyGraph;
pub use executor::{ExecutionResult, ExecutorConfig, PassExecutor};
pub use manager::{PassManager, PassManagerConfig, PassRunReport};
pub use registry::PassRegistry;
pub use scheduler::{ExecutionSchedule, compute_schedule};
