#[macro_use]
extern crate log;

// Hack to print log when running unit test
pub mod ast;

pub use cir;

pub mod analysis;
pub mod irgen;
pub mod parser;
