//! Smart contract IR to facilitate formal verification and symbolic execution.

pub mod block;
pub mod definitions;
pub mod expressions;
pub mod literals;
pub mod source_unit;
pub mod statements;
pub mod types;
pub mod util;
pub mod variable;

pub use block::*;
pub use definitions::*;
pub use expressions::*;
pub use literals::*;
pub use source_unit::*;
pub use statements::*;
pub use types::*;
pub use util::*;
pub use variable::*;

/* Metadata */

pub type Loc = core::metadata::Loc;
pub type DataLoc = core::metadata::DataLoc;
pub type Name = core::metadata::Name;
