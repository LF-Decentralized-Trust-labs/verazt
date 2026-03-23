//! FIR — Functional IR
//!
//! FIR is a functional, tail-recursive IR where every basic block from BIR
//! becomes a function in parameter-passing form. Phi nodes are eliminated
//! into function parameters, and terminators become tail calls.

pub mod lower;
pub mod ops;
pub mod utils;
pub mod verifier;

// Re-exports for convenient access
pub use ops::*;
