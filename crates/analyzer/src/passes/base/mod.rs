//! Pass Abstraction
//!
//! This module defines **what a pass is**: the core traits and
//! the metadata types. It does *not* contain execution machinery
//! (see `pipeline/` for that).
//!
//! Pass identity uses `std::any::TypeId` — no hand-maintained enum.

pub mod meta;
pub mod traits;

// Re-exports for convenience — users can write `use crate::passes::base::Pass;`
pub use meta::{PassLevel, PassRepresentation};
pub use traits::{AnalysisPass, Pass, PassError, PassExecutionInfo, PassResult};
