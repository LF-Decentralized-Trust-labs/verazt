//! Views and abstractions over BIR functions shared by BIR passes and
//! detectors.

pub mod function_view;
pub mod state_access;

pub use function_view::{FunctionView, OpPos};
pub use state_access::StateAccess;
