extern crate core;

#[macro_use]
extern crate log;

pub mod ast;
pub mod globals;
pub mod normalize;
pub mod parsing;
pub mod util;

pub use parsing::parser;
