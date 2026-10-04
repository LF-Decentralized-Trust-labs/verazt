//! Execution Pipeline
//!
//! This module defines **how passes are run**: registration, scheduling,
//! execution, and dependency resolution.
//!
//! ## Responsibility boundaries
//!
//! - **`manager`** — owns the pass registry; entry point for callers; delegates
//!   to scheduler then executor; produces `PassRunReport`.
//! - **`scheduler`** — pure function: takes registered passes, returns
//!   `ExecutionSchedule`; must not mutate `AnalysisContext`.
//! - **`executor`** — takes `ExecutionSchedule` + `AnalysisContext`; drives
//!   execution and timing; must not mutate the pass registry.
//! - **`dependency`** — dependency graph + topological sort.
//! - **`registry`** — constructors of the passes that may be scheduled.

pub mod dependency;
pub mod executor;
pub mod manager;
pub mod registry;
pub mod scheduler;

pub use dependency::DependencyGraph;
pub use executor::{ExecutionResult, ExecutorConfig, PassExecutor};
pub use manager::{PassManager, PassManagerConfig, PassRunReport};
pub use registry::PassRegistry;
pub use scheduler::{ExecutionLevel, ExecutionSchedule, PassScheduler};
