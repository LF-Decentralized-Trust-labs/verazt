//! EVM dialect detectors (Solidity/Vyper)

pub mod contract;
pub mod function;
pub mod module;

// Re-export all detector types
pub use contract::*;
pub use function::*;
pub use module::*;
