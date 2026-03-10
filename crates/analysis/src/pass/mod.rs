//! Pass Abstraction
//!
//! This module defines **what a pass is**: the core traits, the identifier
//! enum, and the metadata types. It does *not* contain execution machinery
//! (see `pipeline/` for that).

pub mod id;
pub mod meta;
pub mod traits;

// Re-exports for convenience — users can write `use analysis::pass::Pass;`
pub use id::{AirPassId, DetectionPassId, PassId, SirPassId};
pub use meta::{PassLevel, PassRepresentation};
pub use traits::{AnalysisPass, Pass, PassError, PassExecutionInfo, PassResult};
