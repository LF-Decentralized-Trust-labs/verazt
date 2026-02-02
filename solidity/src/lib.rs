#[macro_use]
extern crate log;

// Hack to print log when running unit test
pub mod ast;
pub mod compile;
pub mod passes;
pub mod parser;
pub mod version;
