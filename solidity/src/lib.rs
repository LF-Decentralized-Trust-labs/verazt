#[macro_use]
extern crate log;

// Hack to print log when running unit test
pub mod ast;

pub mod ir;

pub mod codegen;
pub mod parser;
pub mod analysis;

