//! Analysis passes for SmartHunt.
//!
//! This module contains the pass infrastructure and implementations
//! for various analysis passes.

pub mod pass;
pub mod symbol_table;
pub mod type_index;
pub mod cfg;
pub mod call_graph;
pub mod data_flow;
pub mod state_mutation;
pub mod access_control;

pub use pass::*;
pub use symbol_table::SymbolTablePass;
pub use type_index::TypeIndexPass;
pub use cfg::CfgPass;
pub use call_graph::CallGraphPass;
pub use data_flow::DataFlowPass;
pub use state_mutation::StateMutationPass;
pub use access_control::AccessControlPass;
